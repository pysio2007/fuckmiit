//! MIIT ICP filing query client.

pub mod api;
mod captcha;
mod error;

use std::time::{SystemTime, UNIX_EPOCH};

use api::{CheckImageResponse, Client, IcpDetailRequest, IcpQueryRequest};

pub use error::{Error, Result};

const DEFAULT_SERVICE_TYPE: i64 = 1;
const CAPTCHA_SEARCH_RADIUS: i32 = 12;

pub type DomainDetail = api::IcpDetailItem;

macro_rules! debugf {
    ($enabled:expr, $($arg:tt)*) => {
        if $enabled {
            eprintln!("debug: {}", format_args!($($arg)*));
        }
    };
}

/// Runs the full MIIT query flow for a domain and returns the raw
/// `queryDetailByServiceIdAndDomainId` JSON response body.
pub fn query_domain(domain: &str, debug: bool) -> Result<Vec<u8>> {
    if domain.is_empty() {
        return Err(Error::EmptyDomain);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before the UNIX epoch")
        .as_secs();
    let mut client = Client::new(api::DEFAULT_HTTP_TIMEOUT)?;

    debugf!(debug, "step=auth timestamp={timestamp}");
    let auth = client.auth(timestamp)?;
    debugf!(
        debug,
        "step=auth success=true expire={}",
        auth.params.expire
    );

    let client_uid = api::new_client_uid()?;
    debugf!(debug, "step=getCheckImagePoint clientUid={client_uid}");

    let challenge = client.get_check_image_point(&client_uid)?.params;
    debugf!(
        debug,
        "step=getCheckImagePoint success=true captchaUUID={} height={}",
        challenge.uuid,
        challenge.height
    );

    let check = solve_captcha(
        &client,
        &challenge.uuid,
        &challenge.big_image,
        challenge.height,
        debug,
    )?;
    client.set_sign(&check.params);
    client.set_uuid(&challenge.uuid);
    debugf!(
        debug,
        "step=query endpoint=icpAbbreviateInfo/queryByCondition unitName={domain} serviceType={DEFAULT_SERVICE_TYPE}"
    );

    let query = client.query_icp_abbreviate_info(&IcpQueryRequest {
        page_num: "",
        page_size: "",
        unit_name: domain,
        service_type: DEFAULT_SERVICE_TYPE,
    })?;
    let list = api::parse_icp_query_list(&query)?;
    let item = list
        .params
        .list
        .first()
        .ok_or_else(|| Error::NoRecords(domain.to_owned()))?;
    debugf!(
        debug,
        "step=queryDetail endpoint=icpAbbreviateInfo/queryDetailByServiceIdAndDomainId mainId={} domainId={} serviceId={}",
        item.main_id,
        item.domain_id,
        item.service_id
    );

    client.query_icp_detail_by_service_id_and_domain_id(&IcpDetailRequest {
        main_id: item.main_id,
        domain_id: item.domain_id,
        service_id: item.service_id,
    })
}

/// Runs the full MIIT query flow and returns the parsed detail record.
pub fn query_domain_detail(domain: &str, debug: bool) -> Result<DomainDetail> {
    parse_domain_detail(&query_domain(domain, debug)?)
}

fn parse_domain_detail(body: &[u8]) -> Result<DomainDetail> {
    let response = api::parse_icp_detail(body)?;
    if !response.success || response.code != 200 {
        return Err(Error::Rejected {
            endpoint: "detail response",
            code: response.code,
            msg: response.msg,
        });
    }
    Ok(response.params)
}

fn solve_captcha(
    client: &Client,
    captcha_uuid: &str,
    big_image: &str,
    top_hint: i32,
    debug: bool,
) -> Result<CheckImageResponse> {
    let gap = captcha::detect_square_base64_with_hint(big_image, top_hint)?;
    debugf!(
        debug,
        "step=detect left={} top={} right={} bottom={}",
        gap.left,
        gap.top,
        gap.right,
        gap.bottom
    );

    for left in candidate_offsets(gap.left, CAPTCHA_SEARCH_RADIUS) {
        debugf!(debug, "step=checkImage attempt_left={left}");
        let response = client.try_check_image(captcha_uuid, left)?;
        if response.code == 200 && response.success {
            debugf!(
                debug,
                "step=checkImage success=true sign_len={}",
                response.params.len()
            );
            return Ok(response);
        }
    }

    Err(Error::CaptchaFailed { left: gap.left })
}

/// `center`, then `center∓1`, `center∓2`, … up to `radius`, skipping negatives.
fn candidate_offsets(center: i32, radius: i32) -> impl Iterator<Item = i32> {
    std::iter::once(center)
        .chain((1..=radius).flat_map(move |delta| [center - delta, center + delta]))
        .filter(|&offset| offset >= 0)
}
