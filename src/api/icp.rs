use serde::{Deserialize, Serialize};

use super::{Client, nullable};
use crate::Result;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpQueryRequest<'a> {
    pub page_num: &'a str,
    pub page_size: &'a str,
    pub unit_name: &'a str,
    pub service_type: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IcpQueryListResponse {
    #[serde(deserialize_with = "nullable")]
    pub code: i64,
    #[serde(deserialize_with = "nullable")]
    pub msg: String,
    #[serde(deserialize_with = "nullable")]
    pub params: IcpQueryListParams,
    #[serde(deserialize_with = "nullable")]
    pub success: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IcpQueryListParams {
    #[serde(deserialize_with = "nullable")]
    pub list: Vec<IcpQueryListItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct IcpQueryListItem {
    #[serde(deserialize_with = "nullable")]
    pub main_id: i64,
    #[serde(deserialize_with = "nullable")]
    pub domain_id: i64,
    #[serde(deserialize_with = "nullable")]
    pub service_id: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IcpDetailRequest {
    pub main_id: i64,
    pub domain_id: i64,
    pub service_id: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct IcpDetailResponse {
    #[serde(deserialize_with = "nullable")]
    pub code: i64,
    #[serde(deserialize_with = "nullable")]
    pub msg: String,
    #[serde(deserialize_with = "nullable")]
    pub params: IcpDetailItem,
    #[serde(deserialize_with = "nullable")]
    pub success: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct IcpDetailItem {
    #[serde(deserialize_with = "nullable")]
    pub content_type_name: String,
    #[serde(deserialize_with = "nullable")]
    pub domain: String,
    #[serde(deserialize_with = "nullable")]
    pub domain_id: i64,
    #[serde(deserialize_with = "nullable")]
    pub leader_name: String,
    #[serde(deserialize_with = "nullable")]
    pub limit_access: String,
    #[serde(deserialize_with = "nullable")]
    pub main_id: i64,
    #[serde(deserialize_with = "nullable")]
    pub main_licence: String,
    #[serde(deserialize_with = "nullable")]
    pub nature_name: String,
    #[serde(deserialize_with = "nullable")]
    pub service_id: i64,
    #[serde(deserialize_with = "nullable")]
    pub service_licence: String,
    #[serde(deserialize_with = "nullable")]
    pub unit_name: String,
    #[serde(deserialize_with = "nullable")]
    pub update_record_time: String,
}

impl Client {
    pub fn query_icp_abbreviate_info(&self, request: &IcpQueryRequest<'_>) -> Result<Vec<u8>> {
        self.post_json_raw("icpAbbreviateInfo/queryByCondition", request)
    }

    pub fn query_icp_detail_by_service_id_and_domain_id(
        &self,
        request: &IcpDetailRequest,
    ) -> Result<Vec<u8>> {
        self.post_json_raw(
            "icpAbbreviateInfo/queryDetailByServiceIdAndDomainId",
            request,
        )
    }
}

pub fn parse_icp_query_list(body: &[u8]) -> Result<IcpQueryListResponse> {
    Ok(serde_json::from_slice(body)?)
}

pub fn parse_icp_detail(body: &[u8]) -> Result<IcpDetailResponse> {
    Ok(serde_json::from_slice(body)?)
}
