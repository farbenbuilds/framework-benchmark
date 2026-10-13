use crate::LoadGenError;

pub(crate) const MAX_RESPONSE_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub status: u16,
    pub len: usize,
    pub closes_connection: bool,
}

pub(crate) fn parse_response(bytes: &[u8]) -> Result<Option<HttpResponse>, LoadGenError> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let (header_len, status) = parse_headers(bytes, &mut headers)?;
    let Some((header_len, status)) = header_len.zip(status) else {
        return Ok(None);
    };
    let closes_connection = header_has_token(&headers, "connection", "close");
    let Some(len) = response_len(bytes, header_len, status, &headers)? else {
        return Ok(None);
    };
    if len > MAX_RESPONSE_BYTES {
        return Err(LoadGenError::Protocol(
            "response exceeds read buffer".into(),
        ));
    }
    Ok(Some(HttpResponse {
        status,
        len,
        closes_connection,
    }))
}

fn parse_headers<'a>(
    bytes: &'a [u8],
    headers: &mut [httparse::Header<'a>],
) -> Result<(Option<usize>, Option<u16>), LoadGenError> {
    let mut response = httparse::Response::new(headers);
    let httparse::Status::Complete(header_len) = response
        .parse(bytes)
        .map_err(|error| LoadGenError::Protocol(error.to_string()))?
    else {
        return Ok((None, None));
    };
    Ok((Some(header_len), response.code))
}

fn response_len(
    bytes: &[u8],
    header_len: usize,
    status: u16,
    headers: &[httparse::Header<'_>],
) -> Result<Option<usize>, LoadGenError> {
    if response_has_no_body(status) {
        return Ok(Some(header_len));
    }
    let content_len = content_length(headers)?;
    let chunked = header_has_token(headers, "transfer-encoding", "chunked");
    if content_len.is_some() && chunked {
        return Err(LoadGenError::Protocol(
            "response includes both content length and chunked encoding".into(),
        ));
    }
    if chunked {
        return parse_chunked_len(bytes, header_len);
    }
    let body_len = content_len
        .ok_or_else(|| LoadGenError::Protocol("response has no self-defined body length".into()))?;
    let len = header_len
        .checked_add(body_len)
        .ok_or_else(|| LoadGenError::Protocol("response length overflows".into()))?;
    Ok((bytes.len() >= len).then_some(len))
}

fn content_length(headers: &[httparse::Header<'_>]) -> Result<Option<usize>, LoadGenError> {
    let mut result = None;
    for header in headers {
        if !header.name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        let value = std::str::from_utf8(header.value)
            .map_err(|error| LoadGenError::Protocol(error.to_string()))?
            .trim()
            .parse::<usize>()
            .map_err(|error| LoadGenError::Protocol(error.to_string()))?;
        if result.is_some_and(|previous| previous != value) {
            return Err(LoadGenError::Protocol("conflicting content lengths".into()));
        }
        result = Some(value);
    }
    Ok(result)
}

fn header_has_token(headers: &[httparse::Header<'_>], name: &str, token: &str) -> bool {
    headers.iter().any(|header| {
        header.name.eq_ignore_ascii_case(name)
            && header
                .value
                .split(|byte| *byte == b',')
                .any(|value| value.trim_ascii().eq_ignore_ascii_case(token.as_bytes()))
    })
}

fn response_has_no_body(status: u16) -> bool {
    (100..200).contains(&status) || status == 204 || status == 304
}

fn parse_chunked_len(bytes: &[u8], header_len: usize) -> Result<Option<usize>, LoadGenError> {
    let mut offset = header_len;
    loop {
        let Some(line_end) = find_crlf(bytes, offset) else {
            return Ok(None);
        };
        let line = &bytes[offset..line_end];
        let size = chunk_size(line)?;
        offset = line_end + 2;
        if size == 0 {
            if bytes.get(offset..offset + 2) == Some(b"\r\n") {
                return Ok(Some(offset + 2));
            }
            let Some(trailer_end) = find_double_crlf(bytes, offset) else {
                return Ok(None);
            };
            return Ok(Some(trailer_end + 4));
        }
        let end = offset
            .checked_add(size)
            .and_then(|value| value.checked_add(2))
            .ok_or_else(|| LoadGenError::Protocol("chunk length overflows".into()))?;
        if end > bytes.len() {
            return Ok(None);
        }
        if &bytes[end - 2..end] != b"\r\n" {
            return Err(LoadGenError::Protocol("chunk lacks CRLF terminator".into()));
        }
        offset = end;
    }
}

fn chunk_size(line: &[u8]) -> Result<usize, LoadGenError> {
    let size = line.split(|byte| *byte == b';').next().unwrap_or_default();
    let size =
        std::str::from_utf8(size).map_err(|error| LoadGenError::Protocol(error.to_string()))?;
    usize::from_str_radix(size.trim(), 16)
        .map_err(|error| LoadGenError::Protocol(error.to_string()))
}

fn find_crlf(bytes: &[u8], offset: usize) -> Option<usize> {
    bytes[offset..]
        .windows(2)
        .position(|pair| pair == b"\r\n")
        .map(|index| offset + index)
}

fn find_double_crlf(bytes: &[u8], offset: usize) -> Option<usize> {
    bytes[offset..]
        .windows(4)
        .position(|value| value == b"\r\n\r\n")
        .map(|index| offset + index)
}

#[cfg(test)]
mod tests {
    use super::parse_response;

    #[test]
    fn parses_content_length_response() {
        let bytes = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
        let response = parse_response(bytes).unwrap().unwrap();
        assert_eq!(response.len, bytes.len());
    }

    #[test]
    fn waits_for_incomplete_content_length_body() {
        let bytes = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\no";
        assert!(parse_response(bytes).unwrap().is_none());
    }

    #[test]
    fn parses_chunked_response_with_trailers() {
        let bytes = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nok\r\n0\r\nX-Test: 1\r\n\r\n";
        let response = parse_response(bytes).unwrap().unwrap();
        assert_eq!(response.len, bytes.len());
    }

    #[test]
    fn parses_chunked_response_without_trailers() {
        let bytes = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nok\r\n0\r\n\r\n";
        assert_eq!(parse_response(bytes).unwrap().unwrap().len, bytes.len());
    }

    #[test]
    fn rejects_ambiguous_response_framing() {
        let bytes = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nTransfer-Encoding: chunked\r\n\r\n";
        assert!(parse_response(bytes).is_err());
    }
}
