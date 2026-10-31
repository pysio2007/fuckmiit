fn main() {
    let domain = "baidu.com";
    match fuckmiit::query_domain_detail(domain, true) {
        Ok(detail) => println!("Query result for {domain}: {}", detail.main_licence),
        Err(err) => println!("Error querying domain: {err}"),
    }
}
