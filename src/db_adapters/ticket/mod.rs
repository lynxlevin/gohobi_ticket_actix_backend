mod ticket_mutation;
mod ticket_query;
pub mod types;
mod wish_mutation;
mod wish_query;

pub use ticket_mutation::TicketMutation;
pub use ticket_query::{Order, TicketQuery};
pub use wish_mutation::WishMutation;
pub use wish_query::WishQuery;
