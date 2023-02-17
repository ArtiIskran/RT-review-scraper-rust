use std::{clone, cmp::Reverse, fs::File, io::Write};

use fantoccini::Client;
use serde::de::value;
use serde_json::Value;

#[tokio::main]
async fn main() {
    scrape_rt_reviews().await;
}

async fn scrape_rt_reviews() {
    // Create client headless browser

    let mut client: Client = match Client::new("http://localhost:9515").await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create client: {:?}", e);
            return;
        }
    };

    //make sure that client is headless
    match client.set_window_rect(0, 0, 0, 0).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to set window rect: {:?}", e);
            return;
        }
    };

    // Navigate to URL
    match client
        .goto("https://www.rottentomatoes.com/m/avengers_endgame/reviews?type=user")
        .await
    {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to navigate to URL: {:?}", e);
            return;
        }
    };

    //retrive javascript varible that stores RottenTomatoes.context.movieReview
    let script = r#"
        var reviews = RottenTomatoes.context.movieReview;
        return reviews;
    "#;

    let movie_data = match client.execute(script, vec![]).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to execute script: {:?}", e);
            return;
        }
    };

    //convert moviedata to json
    let movie_data = match movie_data.as_object() {
        Some(o) => o,
        None => {
            eprintln!("Failed to convert to object");
            return;
        }
    };
    //from movie_data get movieID
    let movie_id = match movie_data.get("movieId") {
        Some(id) => id,
        None => {
            eprintln!("Failed to get movie ID");
            return;
        }
    };
    //from movie_data get pageInfo
    let page_info = match movie_data.get("pageInfo") {
        Some(p) => p,
        None => {
            eprintln!("Failed to get page info");
            return;
        }
    };
    //from page_info get endCursor
    let end_cursor = match page_info.get("endCursor") {
        Some(e) => e,
        None => {
            eprintln!("Failed to get end cursor");
            return;
        }
    };

    //create reviews_api_url
    let reviews_api_url = format!(
        "https://www.rottentomatoes.com/napi/movie/{}/reviews/user?f=null&direction=prev&endCursor={}",
        movie_id, end_cursor
    );
    // from client java script retrive reviews_api_url
    let script = format!(
        r#"
        var reviews = RottenTomatoes.context.movieReview;
        var movieID = reviews.movieId;
        var page_info = reviews.pageInfo;
        var end_cursor = page_info.endCursor;
        var reviews_api_url = "https://www.rottentomatoes.com/napi/movie/" + movieID + "/reviews/user?f=null&direction=prev&endCursor=" + end_cursor;
        return reviews_api_url;
    "#
    );
    let reviews_api_url = match client.execute(&script, vec![]).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to execute script: {:?}", e);
            return;
        }
    };
    //get data from that url
    let reviews_api_url = match reviews_api_url.as_str() {
        Some(s) => s,
        None => {
            eprintln!("Failed to convert to string");
            return;
        }
    };
    //use reqest to get data from that url
    let res = match reqwest::get(reviews_api_url).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to get response: {:?}", e);
            return;
        }
    };
    //print
    let page_one_items = res.text().await.unwrap();

    //convert to json
    let page_one_items: serde_json::Value = match serde_json::from_str(&page_one_items) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to convert to JSON: {:?}", e);
            return;
        }
    };
    //ist we will add all fetched reviews to
    let mut all_items = vec![];
    //add reveiws to all reviews
    let items = page_one_items.clone();
    all_items.push(page_one_items);
    //from page one items get pageInfo
    let mut page_info = match items.get("pageInfo") {
        Some(p) => p,
        None => {
            eprintln!("Failed to get page info");
            return;
        }
    };
    //get has next page
    let mut has_next_page = match page_info.get("hasNextPage") {
        Some(h) => h,
        None => {
            eprintln!("Failed to get has next page");
            return;
        }
    };
    //get end cursor
    let mut end_cursor = match page_info.get("endCursor") {
        Some(e) => e,
        None => {
            eprintln!("Failed to get end cursor");
            return;
        }
    };

    let Client = &client;
    //while has next page is true grb more reviews
    while has_next_page.as_bool().unwrap() {
        //create reviews_api_url
        let reviews_api_url = format!(
            "https://www.rottentomatoes.com/napi/movie/{}/reviews/user?f=null&direction=prev&endCursor={}",
            movie_id, end_cursor
        );
        // from client java script retrive reviews_api_url
        let script = format!(
            r#"
        var reviews = RottenTomatoes.context.movieReview;
        var movieID = reviews.movieId;
        var page_info = reviews.pageInfo;
        var end_cursor = page_info.endCursor;
        var reviews_api_url = "https://www.rottentomatoes.com/napi/movie/" + movieID + "/reviews/user?f=null&direction=prev&endCursor=" + end_cursor;
        return reviews_api_url;"#
        );
        //use reqeust to execute script
        let reviews_api_url = match Client.execute(&script, vec![]).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to execute script: {:?}", e);
                return;
            }
        };
        //get data from that url
        let reviews_api_url = match reviews_api_url.as_str() {
            Some(s) => s,
            None => {
                eprintln!("Failed to convert to string");
                return;
            }
        };
        //use reqest to get data from that url
        let res = match reqwest::get(reviews_api_url).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to get response: {:?}", e);
                return;
            }
        };
        //print
        let items = res.text().await.unwrap();
        //convert to json
        let items_json: serde_json::Value = match serde_json::from_str(&items) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to convert to JSON: {:?}", e);
                return;
            }
        };
        //add reveiws to all reviews
        all_items.push(items_json);
print!("{:?}",all_items);
        //get has next page
        has_next_page = match page_info.get("hasNextPage") {
            Some(h) => h,
            None => {
                eprintln!("Failed to get has next page");
                return;
            }
        };
        //get end cursor
        end_cursor = match page_info.get("endCursor") {
            Some(e) => e,
            None => {
                eprintln!("Failed to get end cursor");
                return;
            }
        };
    }
let mut reveiws:&Vec<Value>= &vec![];
    //foreach all items get reviews
    //create list reveiews
    let mut reviews_list = vec![];
    for item in all_items {
        let reviews = match item.get("reviews").cloned() {
            Some(r) => r,
            None => {
                eprintln!("Failed to get reviews");
                return;
            }
        };
        //add reveiws to list
        reviews_list.push(reviews);

    }
    //list of reveiws
    let mut reveiws = vec![];
    //fore each reveiws get review
    for review in reviews_list {
        //get review
        let review = match review.get("review").cloned() {
            Some(r) => r,
            None => {
                eprintln!("Failed to get review");
                return;
            }
        };
        //add reveiw to list
        reveiws.push(review);
    }
    //save reviews to json
    let mut file = File::create("reviews.json");
    match file {
        Ok(mut f) => {
            let json = serde_json::to_string_pretty(&reveiws).unwrap();
            f.write_all(json.as_bytes()).unwrap();
        }
        Err(e) => {
            eprintln!("Failed to create file: {:?}", e);
            return;
        }
    }

}
