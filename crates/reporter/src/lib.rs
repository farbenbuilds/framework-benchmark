pub mod archive;
pub mod chart;
pub mod error;
pub mod index;
pub mod logic;
pub mod markdown;

pub use archive::{timestamped_result_path, timestamped_result_relative_path, write_result_json};
pub use chart::generate_throughput_svg;
pub use error::ReporterError;
pub use index::{HistoricalIndex, IndexEntry, update_historical_index};
pub use logic::{LeaderboardRow, aggregate_leaderboard};
pub use markdown::{render_markdown_table, render_readme_leaderboards, update_readme_leaderboard};
