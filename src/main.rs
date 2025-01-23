#[macro_use]
extern crate rocket;

use cors::CORS;
use routes::zipf::zipf_plot;

mod cors;
mod routes;
mod utils;

#[launch]
fn rocket() -> _ {
    rocket::build().attach(CORS).mount("/", routes![zipf_plot])
}
