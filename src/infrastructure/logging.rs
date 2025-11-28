use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::{layer::Context, Layer};

pub struct SqlxLogLayer;

impl<S> Layer<S> for SqlxLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Check if the event is from sqlx
        let metadata = event.metadata();
        if !metadata.target().starts_with("sqlx::query") {
            return;
        }

        // Create a visitor to extract fields
        let mut visitor = SqlxEventVisitor::default();
        event.record(&mut visitor);

        // Format the log
        if let Some(query) = visitor.query {
            let duration = visitor.elapsed.unwrap_or_default();

            // Determine if it's a query or execute result
            if let Some(rows) = visitor.rows_returned {
                println!(
                    "\n  ├─ 🗄️ SQL Query\n  │  {}\n  │  Duration: {} | Rows: {}",
                    query.trim(),
                    duration,
                    rows
                );
            } else if let Some(affected) = visitor.rows_affected {
                println!(
                    "\n  ├─ 🗄️ SQL Execute\n  │  {}\n  │  Duration: {} | Affected: {}",
                    query.trim(),
                    duration,
                    affected
                );
            } else {
                // Fallback for other events
                println!("\n  ├─ 🗄️ SQL Event\n  │  {}\n  │  Duration: {}", query.trim(), duration);
            }
        }
    }
}

#[derive(Default)]
struct SqlxEventVisitor {
    query: Option<String>,
    rows_affected: Option<u64>,
    rows_returned: Option<u64>,
    elapsed: Option<String>,
}

impl tracing::field::Visit for SqlxEventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        match field.name() {
            "summary" => {
                if self.query.is_none() {
                    self.query = Some(format!("{:?}", value).trim_matches('"').to_string())
                }
            },
            "db.statement" => {
                self.query = Some(format!("{:?}", value).trim_matches('"').to_string())
            },
            "rows_affected" => {
                self.rows_affected = Some(format!("{:?}", value).parse().unwrap_or(0))
            },
            "rows_returned" => {
                self.rows_returned = Some(format!("{:?}", value).parse().unwrap_or(0))
            },
            "elapsed" => self.elapsed = Some(format!("{:?}", value)),
            _ => {},
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "rows_affected" => self.rows_affected = Some(value),
            "rows_returned" => self.rows_returned = Some(value),
            _ => {},
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "summary" => {
                if self.query.is_none() {
                    self.query = Some(value.to_string())
                }
            },
            "db.statement" => self.query = Some(value.to_string()),
            "elapsed" => self.elapsed = Some(value.to_string()),
            _ => {},
        }
    }
}
