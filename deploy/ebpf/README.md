# eBPF metrics

Run the collector with the target container or process PID for the same interval as the benchmark measurement:

```sh
sudo scripts/collect_ebpf.sh 1234 30 results/ebpf.txt
```

It emits off CPU microseconds, page cache hit and miss events, and context switches. The collector crate parses this stable text format and calculates the page cache hit ratio.

The host needs `bpftrace`, BTF support, and permissions to attach scheduler tracepoints and the available page cache probes.
