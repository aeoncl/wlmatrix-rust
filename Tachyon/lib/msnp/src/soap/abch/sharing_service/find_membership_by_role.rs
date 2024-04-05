pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::ab_service::ab_contact_update::request::Options;
    use crate::soap::abch::msnab_datatypes::{ArrayOfRoleId, ServiceFilter};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipByRoleMessage {
        #[yaserde(rename = "FindMembershipByRole", default)]
        pub body: FindMembershipByRoleRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipByRoleRequestType")]
    pub struct FindMembershipByRoleRequestType {
        #[yaserde(rename = "serviceFilter", default)]
        pub service_filter: Option<ServiceFilter>,
        #[yaserde(rename = "includedRoles", default)]
        pub included_roles: Option<ArrayOfRoleId>,
        #[yaserde(rename = "view", default)]
        pub view: String,
        #[yaserde(rename = "expandMembership", default)]
        pub expand_membership: bool,
        #[yaserde(rename = "options", default)]
        pub options: Option<Options>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindMembershipByRoleMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipByRoleMessage,
    }

    impl FindMembershipByRoleMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipByRoleMessage) -> Self {
            FindMembershipByRoleMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::sharing_service::find_membership::response::MembershipResult;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipByRoleResponseMessage {
        #[yaserde(rename = "FindMembershipByRoleResponseMessage", default)]
        pub body: FindMembershipByRoleResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "FindMembershipByRoleResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct FindMembershipByRoleResponse {
        #[yaserde(rename = "FindMembershipByRoleResult", default)]
        pub find_membership_by_role_result: MembershipResult,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindMembershipByRoleResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipByRoleResponseMessage,
    }

    impl FindMembershipByRoleResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipByRoleResponseMessage) -> Self {
            FindMembershipByRoleResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}