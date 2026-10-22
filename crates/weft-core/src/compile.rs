// SPDX-License-Identifier: MIT OR Apache-2.0
//! Public, pure compile transport. Hosts own backend registration and execution.
/// Maximum UTF-8 request bytes admitted by the compiler and CLI.
pub const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
use crate::{
    backend::{BindingInput, Plan, Registry, Representation, Status, Target},
    error::{Diagnostic, Result},
    model::ModuleInput,
};
use serde::Deserialize;
use serde_json::{json, Value};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    interface_version: String,
    dialect: String,
    sql: String,
    modules: Vec<ModuleInput>,
    target: TargetRequest,
    #[serde(default)]
    options: Options,
    parameters: Option<crate::application_resolve::Parameters>,
    exact_parameters:Option<crate::application_resolve::ExactParameters>,
    read_profile: Option<crate::application_ir::ReadProfile>,
    timestamp_comparison_profile: Option<String>,
    security: Option<Value>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TargetRequest {
    backend_id: String,
    backend_version: String,
    target_profile: String,
    binding_json: String,
    binding_sha256: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    #[serde(default)]
    allow_candidate: bool,
}
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request.schema.json")]
struct Request01;
#[jsonschema::validator(
    path = "../../docs/helix/02-design/contracts/compile-request-v0.2.schema.json"
)]
struct Request02;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.1.schema.json")]
struct Request021;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.2.schema.json")]
struct Request022;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.2.schema.json")]
struct Plan022;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.3.schema.json")]
struct Request023;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.3.schema.json")]
struct Plan023;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.4.schema.json")]
struct Request024;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.4.schema.json")]
struct Plan024;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.5.schema.json")]
struct Request025;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.5.schema.json")]
struct Plan025;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.6.schema.json")]
struct Request026;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.6.schema.json")]
struct Plan026;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.7.schema.json")]
struct Request027;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.7.schema.json")]
struct Plan027;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.2.8.schema.json")]
struct Request028;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/logical-plan-v0.2.8.schema.json")]
struct Plan028;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.3.schema.json")]
struct Request03;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/security-compile-request-v0.1.schema.json")]
struct RequestSecurity01;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/security-compile-request-v0.2.schema.json")]
struct RequestSecurity02;
#[jsonschema::validator(path = "../../docs/helix/02-design/contracts/compile-request-v0.5.schema.json")]
struct RequestSecurityCompat05;
/// Validated request target supplied to trusted host composition code.
/// Binding bytes/digest, modules and SQL/plan have passed transport admission.
pub struct CompositionInput<'a> {
    pub backend_id: &'a str,
    pub backend_version: &'a str,
    pub target_profile: &'a str,
    pub binding_json: &'a str,
    pub binding_sha256: &'a str,
    pub allow_candidate: bool,
}
/// Called once per valid resolved request. Models cannot select executable code.
/// Hosts provide pure admission/registration; execution and IO stay outside.
pub type RegistryFactory<'a> =
    dyn FnMut(&crate::model::Catalog, Plan<'_>, CompositionInput<'_>) -> Result<Registry> + 'a;
#[derive(Default)]
pub struct Compiler {
    pub registry: Registry,
}
impl Compiler {
    pub fn compile_json(&self, raw: &str) -> String {
        self.compile_json_internal(raw, None, None, false, None, false, false)
    }
    pub fn compile_json_with_factory(
        &self,
        raw: &str,
        factory: &mut RegistryFactory<'_>,
    ) -> String {
        self.compile_json_internal(raw, Some(factory), None, false, None, false, false)
    }
    /// Only a trusted embedding selects this additional numeric interpretation.
    pub fn compile_json_with_exact_numbers_factory(&self,raw:&str,factory:&mut RegistryFactory<'_>)->String{
        self.compile_json_internal(raw,Some(factory),None,true, None, false, false)
    }
    pub fn compile_json_with_security_factory(
        &self,
        raw: &str,
        factory: &mut crate::security_backend::SecurityRegistryFactory<'_>,
    ) -> String {
        self.compile_json_internal(raw, None, Some(factory), false, None, false, false)
    }
    /// Explicit host-selected authored-key page composition; ordinary compilation is unchanged.
    pub fn compile_page_json_with_exact_numbers_factory(
        &self, raw: &str, page_size: u16, after: Option<Vec<String>>,
        factory: &mut RegistryFactory<'_>,
    ) -> String {
        self.compile_json_internal(raw, Some(factory), None, true, Some((page_size, after, false, false)), false, false)
    }
    pub fn compile_typed_page_json_with_exact_numbers_factory(&self,raw:&str,page_size:u16,after:Option<Vec<String>>,factory:&mut RegistryFactory<'_>) -> String {
        self.compile_json_internal(raw,Some(factory),None,true,Some((page_size,after,true,false)), false, false)
    }

    /// Explicit decimal(12,2) key extension; existing page factories stay closed.
    pub fn compile_decimal12_typed_page_json_with_exact_numbers_factory(&self,raw:&str,page_size:u16,after:Option<Vec<String>>,factory:&mut RegistryFactory<'_>) -> String {
        self.compile_json_internal(raw,Some(factory),None,true,Some((page_size,after,true,true)), true, false)
    }

    /// Explicit name-or-stable-ID profile; legacy factories remain name-only.
    pub fn compile_identity_names_json_with_exact_numbers_factory(&self,raw:&str,factory:&mut RegistryFactory<'_>)->String {
        self.compile_json_internal(raw,Some(factory),None,true,None,true, false)
    }

    /// Explicit cursor delivery over one authored-Key source. Hosts own fetching
    /// and snapshot/current-authority enforcement; ordinary factories stay bounded.
    pub fn compile_cursor_json_with_exact_numbers_factory(&self,raw:&str,batch_size:u16,factory:&mut RegistryFactory<'_>)->String {
        self.compile_json_internal(raw,Some(factory),None,true,Some((batch_size,None,true,true)),true,true)
    }

    /// Compose source entity selection through an original canonical relationship.
    /// Source and target share this request's admitted Catalog, binding and parameters.
    pub fn compile_navigation_json_with_factory(&self,raw:&str,navigation:&crate::application_navigation::Input,factory:&mut RegistryFactory<'_>,exact_numbers:bool,identity_names:bool,page:Option<(u16,Option<Vec<String>>,bool)>)->String{
        let cursor=page.as_ref().is_some_and(|(_,_,cursor)|*cursor);
        let decimal12=crate::json::checked_json(raw).is_ok_and(|r|matches!(r["interfaceVersion"].as_str(),Some("weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8")));
        self.compile_json_navigation_internal(raw,Some(factory),None,exact_numbers,page.map(|(size,after,_)|(size,after,true,decimal12)),identity_names,cursor,Some(navigation),false,false,None,None,None)
    }
    fn compile_json_internal(
        &self,
        raw: &str,
        factory: Option<&mut RegistryFactory<'_>>,
        security_factory: Option<&mut crate::security_backend::SecurityRegistryFactory<'_>>,
        exact_numbers: bool,
        page: Option<(u16, Option<Vec<String>>, bool, bool)>,
        identity_name_fallback: bool,
        cursor_delivery: bool,
    ) -> String {
        self.compile_json_navigation_internal(raw,factory,security_factory,exact_numbers,page,identity_name_fallback,cursor_delivery,None,false,false,None,None,None)
    }
    /// Trusted original direct-Field profile; parameters and ordinary profiles stay closed.
    pub fn compile_field_projection_json(&self,raw:&str)->String {
        self.compile_json_navigation_internal(raw,None,None,false,None,false,false,None,true,false,None,None,None)
    }
    /// Trusted additive scalar-presence profile. Original source bytes stay intact.
    pub fn compile_sdk_presence_json(&self,raw:&str)->String {
        self.compile_json_navigation_internal(raw,None,None,false,None,false,false,None,false,true,None,None,None)
    }
    #[cfg(feature="engine-ordered-page-candidate")]
    pub fn compile_sdk_ordered_json(&self,raw:&str,selected:&crate::application_ordered_page::SelectedKey,page_size:u16,after:Option<crate::application_ordered_page::Cursor>,navigation:Option<&crate::application_navigation::Input>,source_selected_key:Option<&crate::application_ordered_page::SelectedKey>)->String {
        let page=json!({"selectedKey":selected,"pageSize":page_size,"after":after});
        self.compile_json_navigation_internal(raw,None,None,false,None,false,false,navigation,false,true,Some(&page),source_selected_key.map(|s|json!(s)).as_ref(),None)
    }
    #[cfg(feature="engine-ordered-page-candidate")]
    pub fn compile_sdk_navigation_json(&self,raw:&str,navigation:&crate::application_navigation::Input,source_selected_key:&crate::application_ordered_page::SelectedKey)->String {
        self.compile_json_navigation_internal(raw,None,None,false,None,false,false,Some(navigation),false,true,None,Some(&json!(source_selected_key)),None)
    }
    #[cfg(feature="engine-ordered-page-candidate")]
    pub fn compile_sdk_read_json(&self,raw:&str,order_key:Option<&crate::application_ordered_page::SelectedKey>,navigation:Option<&crate::application_navigation::Input>,source_key:Option<&crate::application_ordered_page::SelectedKey>)->String{
      self.compile_json_navigation_internal(raw,None,None,false,None,false,false,navigation,false,true,None,source_key.map(|s|json!(s)).as_ref(),order_key.map(|s|json!(s)).as_ref())
    }
    fn compile_json_navigation_internal(
        &self,
        raw: &str,
        factory: Option<&mut RegistryFactory<'_>>,
        security_factory: Option<&mut crate::security_backend::SecurityRegistryFactory<'_>>,
        exact_numbers: bool,
        page: Option<(u16, Option<Vec<String>>, bool, bool)>,
        identity_name_fallback: bool,
        cursor_delivery: bool,
        navigation:Option<&crate::application_navigation::Input>,
        field_projection_decimal38:bool,
        sdk_scalar_presence:bool,
        ordered_page:Option<&Value>,
        sdk_source_key:Option<&Value>,
        sdk_order_key:Option<&Value>,
    ) -> String {
        #[cfg(not(feature="engine-ordered-page-candidate"))]
        if ordered_page.is_some()||sdk_order_key.is_some(){return json!({"status":"blocked","diagnostics":[{"code":"WFT-ORDERED-PAGE","message":"Original ordered page feature not selected"}]}).to_string();}
        let mut version = "weft-compile/0.1.0".to_string();
        let mut diagnostic_sql=None;
        let result = (|| -> Result<Value> {
            if let Some((size, after, _, _)) = page.as_ref() {
                if !(1..=999).contains(size) || after.as_ref().is_some_and(|items| items.len()>1024 || items.iter().try_fold(0usize, |total, value| total.checked_add(value.len())).is_none_or(|bytes| bytes>65536)) {
                    return Err(Diagnostic::new("WFT-LIMIT", "input", "Page size or continuation exceeds its bounds"));
                }
            }
            if raw.len() > MAX_REQUEST_BYTES {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "input",
                    "Request exceeds sixteen MiB",
                ));
            }
            let input = crate::json::checked_json(raw)
                .map_err(|code| Diagnostic::new(code, "input", "Invalid request JSON"))?;
            if !input["interfaceVersion"].is_string() || !input["dialect"].is_string() {
                return Err(Diagnostic::new(
                    "WFT-INPUT",
                    "input",
                    "Compile and dialect versions must be supplied strings",
                ));
            }
            if let Some(v @ ("weft-compile/0.2.0" | "weft-compile/0.2.1" | "weft-compile/0.2.2" | "weft-compile/0.2.3" | "weft-compile/0.2.4" | "weft-compile/0.2.5" | "weft-compile/0.2.6" | "weft-compile/0.2.7" | "weft-compile/0.2.8" | "weft-compile/0.3.0" | "weft-security-compile/0.1.0" | "weft-security-compile/0.2.0" | "weft-compile/0.5.0")) = input["interfaceVersion"].as_str() {
                version = v.into();
            }
            if security_factory.is_some() && version != "weft-security-compile/0.2.0" {
                return Err(Diagnostic::new("WFT-VERSION", "input", "Security registration entrypoint requires weft-security-compile/0.2.0"));
            }
            if !matches!(
                (
                    input["interfaceVersion"].as_str(),
                    input["dialect"].as_str()
                ),
                (Some("weft-compile/0.1.0"), Some("weft-sql/0.1.0"))
                    | (Some("weft-compile/0.2.0"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.1"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.2"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.4"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.5"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.6"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.7"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.8"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.2.3"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.3.0"), Some("weft-sql/0.3.0"))
                    | (Some("weft-security-compile/0.1.0"), Some("weft-sql/0.2.0"))
                    | (Some("weft-security-compile/0.2.0"), Some("weft-sql/0.2.0"))
                    | (Some("weft-compile/0.5.0"), Some("weft-sql/0.2.0"))
            ) {
                return Err(Diagnostic::new(
                    "WFT-VERSION",
                    "input",
                    "Unsupported or mismatched compile/dialect versions",
                ));
            }
            if !(if version == "weft-compile/0.1.0" {
                Request01::is_valid(&input)
            } else if version == "weft-compile/0.5.0" {
                RequestSecurityCompat05::is_valid(&input)
            } else if version == "weft-security-compile/0.2.0" {
                RequestSecurity02::is_valid(&input)
            } else if version == "weft-security-compile/0.1.0" {
                RequestSecurity01::is_valid(&input)
            } else if version == "weft-compile/0.2.8" { Request028::is_valid(&input)
            } else if version == "weft-compile/0.2.7" { Request027::is_valid(&input)
            } else if version == "weft-compile/0.2.6" { Request026::is_valid(&input)
            } else if version == "weft-compile/0.2.5" { Request025::is_valid(&input)
            } else if version == "weft-compile/0.2.4" { Request024::is_valid(&input)
            } else if version == "weft-compile/0.2.3" { Request023::is_valid(&input)
            } else if version == "weft-compile/0.2.2" {
                Request022::is_valid(&input)
            } else if version == "weft-compile/0.2.1" {
                Request021::is_valid(&input)
            } else if version == "weft-compile/0.2.0" {
                Request02::is_valid(&input)
            } else {
                Request03::is_valid(&input)
            }) {
                return Err(Diagnostic::new(
                    "WFT-INPUT",
                    "input",
                    "Request violates its versioned envelope",
                ));
            }
            if page.is_some() && !matches!(version.as_str(),"weft-compile/0.2.2"|"weft-compile/0.2.3"|"weft-compile/0.2.4"|"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8") {
                return Err(Diagnostic::new("WFT-VERSION", "input", "SQL page composition requires compile/0.2.2"));
            }
            let mut req: Request = serde_json::from_value(input)
                .map_err(|_| Diagnostic::new("WFT-INPUT", "input", "Malformed request members"))?;
            if navigation.is_some()&&!matches!(req.interface_version.as_str(),"weft-compile/0.2.0"|"weft-compile/0.2.1"|"weft-compile/0.2.2"|"weft-compile/0.2.3"|"weft-compile/0.2.4"|"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8"){return Err(Diagnostic::new("WFT-ENGINE-NAVIGATION","input","Original application source/target compile profile required"));}
            let timestamp_equality=matches!(req.interface_version.as_str(),"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8") && req.timestamp_comparison_profile.as_deref()==Some(crate::exact_instant::PROFILE);
            if req.sql.len() > 65536 {
                return Err(Diagnostic::new("WFT-LIMIT", "input", "SQL exceeds 64 KiB"));
            }
            if version=="weft-compile/0.2.8" {diagnostic_sql=Some(req.sql.clone());}
            if req.target.binding_json.len() > 4 * 1024 * 1024 {
                return Err(Diagnostic::new(
                    "WFT-LIMIT",
                    "input",
                    "Binding exceeds four MiB",
                ));
            }
            if crate::json::sha256(req.target.binding_json.as_bytes()) != req.target.binding_sha256
            {
                return Err(Diagnostic::new(
                    "WFT-PIN",
                    "input",
                    "Binding byte digest mismatch",
                ));
            }
            let truss_runtime_catalog = req.target.backend_id == "truss.postgresql.runtime"
                && ((req.interface_version == "weft-compile/0.2.7" && req.target.backend_version == "0.4.41" && req.target.target_profile == "truss-runtime-pg18.6/0.5")
                 || (req.interface_version == "weft-compile/0.2.8" && matches!(req.target.backend_version.as_str(),"0.4.43"|"0.4.44") && matches!(req.target.target_profile.as_str(),"truss-runtime-pg18.6/0.6"|"truss-runtime-pg18.6-export/0.2")));
            (if truss_runtime_catalog {
                crate::json::checked_truss_runtime_catalog_json(&req.target.binding_json)
            } else {
                crate::json::checked_json(&req.target.binding_json)
            })
                .map_err(|code| Diagnostic::new(code, "input", "Invalid supplied binding JSON"))?;
            if field_projection_decimal38 {
                let query=crate::application_syntax::parse(&req.sql)?;
                if req.interface_version!="weft-compile/0.2.0" || req.dialect!="weft-sql/0.2.0"
                    || req.target.backend_id!="ashlar.databricks" || req.target.backend_version!="0.1.0-local-spark4-fields-candidate"
                    || req.target.target_profile!="spark4.0.1-delta4.0.0-fields-candidate"
                    || exact_numbers || navigation.is_some() || page.is_some() || cursor_delivery
                    || security_factory.is_some() || req.security.is_some() || req.read_profile.is_some()
                    || req.timestamp_comparison_profile.is_some() || req.exact_parameters.is_some()
                    || req.parameters.as_ref().is_some_and(|p|!p.is_empty())
                    || !query.predicates.is_empty() || !query.joins.is_empty() || !query.groups.is_empty()
                    || query.limit.is_none() || query.order.len()!=1
                    || query.outputs.is_empty() || query.outputs.iter().any(|p|!matches!(p.output,crate::application_syntax::Output::Field(_))) {
                    return Err(Diagnostic::new("WFT-FIELD-PROFILE","input","Direct required Field-only profile refuses broader semantics"));
                }
            }
            if sdk_source_key.is_some() != (sdk_scalar_presence && navigation.is_some()){return Err(Diagnostic::new("WFT-ENGINE-NAVIGATION","input","Original SDK source selected Key custody required"));}
            if sdk_scalar_presence && (req.modules.is_empty()||req.modules.iter().any(|m|m.pin.umf_version!="0.10.0")||req.interface_version!="weft-compile/0.2.0" || req.dialect!="weft-sql/0.2.0" || req.target.backend_id!="ashlar.databricks" || !matches!(req.target.backend_version.as_str(),"0.1.0-local-spark4-sdk-candidate"|"0.1.0-local-spark4-sdk-pages-candidate") || req.target.target_profile!="spark4.0.1-delta4.0.0-sdk-candidate" || (ordered_page.is_some() && req.target.backend_version!="0.1.0-local-spark4-sdk-pages-candidate") || page.is_some() || cursor_delivery || field_projection_decimal38 || req.security.is_some() || req.exact_parameters.is_some()) {
                return Err(Diagnostic::new("WFT-SDK-PRESENCE","input","Explicit non-temporal SDK presence profile required"));
            }
            let mut catalog = if matches!(req.interface_version.as_str(), "weft-security-compile/0.1.0" | "weft-security-compile/0.2.0" | "weft-compile/0.5.0") {
                crate::model::Catalog::prepare_security(req.modules)?
            } else if truss_runtime_catalog {
                crate::model::Catalog::prepare_truss_runtime(req.modules, exact_numbers)?
            } else if exact_numbers {
                crate::model::Catalog::prepare_exact_numbers(req.modules)?
            } else {
                crate::model::Catalog::prepare(req.modules)?
            };
            catalog.identity_name_fallback = identity_name_fallback;
            catalog.field_projection_decimal38=field_projection_decimal38;
            catalog.sdk_scalar_presence=sdk_scalar_presence;
            catalog.exact_scalars = matches!(req.interface_version.as_str(), "weft-compile/0.2.1" | "weft-compile/0.2.2" | "weft-compile/0.2.3" | "weft-compile/0.2.4" | "weft-compile/0.2.5" | "weft-compile/0.2.6" | "weft-compile/0.2.7"|"weft-compile/0.2.8");
            catalog.exact_dates = matches!(req.interface_version.as_str(),"weft-compile/0.2.7"|"weft-compile/0.2.8");
            if let Some(source)=req.security.as_ref() {
                let packet=crate::security_source::SecuritySourcePacket::read(
                    source["policyJson"].as_str().expect("security request schema"),
                    source["ontologyJson"].as_str().expect("security request schema"),&catalog)?;
                let security_plan=crate::security_ir::SecurityLogicalPlan::read(packet,&catalog)?;
                if let Some(profile)=source.get("queryProfileJson"){
                    let profile=crate::security_query_profile::SecurityQueryProfile::read(profile.as_str().expect("security request schema"),&security_plan,&catalog,&req.target.binding_json,&req.target.backend_id,&req.target.backend_version,&req.target.target_profile)?;
                    let query=crate::security_query_uses::SecurityResolvedQuery::resolve(&req.sql,&catalog,&security_plan,req.parameters.clone().unwrap_or_default(),req.read_profile.clone())?;
                    let uses=profile.admit_resolved_query(&query,&security_plan,&catalog,&req.target.binding_json,&req.target.backend_id,&req.target.backend_version,&req.target.target_profile)?;
                    if req.interface_version == "weft-security-compile/0.2.0" {
                        let factory=security_factory.ok_or_else(|| Diagnostic::new("WFT-SECURITY-BACKEND-REQUIRED", "capability", "Explicit security registration is required"))?;
                        let context=crate::security_backend::SecurityBackendContext::new(&catalog,&security_plan,&query,&uses,&req.target.binding_json,&req.target.backend_id,&req.target.backend_version,&req.target.target_profile)?;
                        let registry=factory(&context).map_err(|d| crate::security_backend::callback_error(d,req.sql.len()))?;
                        registry.select(&context)?;
                        uses.require_sources(&security_plan,&catalog,&req.target.binding_json,&req.target.backend_id,&req.target.backend_version,&req.target.target_profile)?;
                        // Registration does not admit returned SQL or a native installation.
                        // Keep pure lowering closed until independently complete coverage,
                        // source-domain and result-contract validation is implemented.
                        return Err(Diagnostic::new("WFT-SECURITY-LOWERING-UNSUPPORTED", "capability", "Security physical/result/dependency coverage validation is not implemented"));
                    }
                }
                return Err(Diagnostic::new("WFT-SECURITY-UNSUPPORTED", "capability", "Security policy lowering and native host obligations are not implemented"));
            }
            if req.interface_version == "weft-compile/0.1.0" {
                let query = crate::syntax::parse(&req.sql)?;
                let plan = crate::resolve::resolve(&catalog, query)?;
                self.emit(
                    &req.interface_version,
                    &req.dialect,
                    &catalog,
                    Plan::V01(&plan),
                    req.target,
                    req.options.allow_candidate,
                    factory,
                )
            } else if req.interface_version == "weft-compile/0.3.0" {
                let query = crate::arithmetic_query::parse(&req.sql)?;
                let plan = crate::arithmetic_application_resolve::resolve(
                    &catalog, query, req.parameters.unwrap_or_default(), None,
                )?;
                self.emit(
                    &req.interface_version, &req.dialect, &catalog, Plan::V03(&plan),
                    req.target, req.options.allow_candidate, factory,
                )
            } else {
                #[cfg(feature="engine-ordered-page-candidate")]
                let mut query=if ordered_page.is_some()||sdk_scalar_presence{crate::application_syntax::parse_security(&req.sql)?.0}else{crate::application_syntax::parse(&req.sql)?};
                #[cfg(not(feature="engine-ordered-page-candidate"))]
                let query = crate::application_syntax::parse(&req.sql)?;
                #[cfg(feature="engine-ordered-page-candidate")]
                if sdk_scalar_presence && ordered_page.is_none() && ((!query.order.is_empty()) != sdk_order_key.is_some()) {return Err(Diagnostic::new("WFT-ORDERED-READ","input","Original ordered read requires orderKey exactly when ORDER BY is authored"));}
                let mut navigation_parameters=req.parameters.clone().unwrap_or_default();let mut navigation_exact=req.exact_parameters.clone().unwrap_or_default();
                if let Some(input)=navigation{
                    let source=if sdk_scalar_presence{crate::application_syntax::parse_security(&input.source_sql)?.0}else{crate::application_syntax::parse(&input.source_sql)?};
                    let partition=crate::application_navigation::partition(&source,&query,navigation_parameters,navigation_exact)?;
                    navigation_parameters=partition.source;navigation_exact=partition.source_exact;req.parameters=Some(partition.target);req.exact_parameters=Some(partition.target_exact);
                }

                #[cfg(feature="engine-ordered-page-candidate")]
                if let Some(input)=ordered_page {
                    use crate::application_ordered_page::{SelectedKey,Cursor};
                    let selected:SelectedKey=serde_json::from_value(input["selectedKey"].clone()).map_err(|_|Diagnostic::new("WFT-ORDERED-PAGE","input","Closed original selected Key required"))?;
                    let size=input["pageSize"].as_u64().and_then(|v|u16::try_from(v).ok()).ok_or_else(||Diagnostic::new("WFT-ORDERED-PAGE","input","Bounded original page size required"))?;
                    let after:Option<Cursor>=serde_json::from_value(input["after"].clone()).map_err(|_|Diagnostic::new("WFT-ORDERED-PAGE","input","Closed full typed cursor required"))?;
                    let mut ordered=crate::application_ordered_page::prepare(&catalog,&req.sql,req.parameters.clone().unwrap_or_default(),&selected,size,after)?;
                    if let Some(input)=navigation {
                        let selected:crate::application_ordered_page::SelectedKey=serde_json::from_value(sdk_source_key.ok_or_else(||Diagnostic::new("WFT-ENGINE-NAVIGATION","input","Original source selected Key required"))?.clone()).map_err(|_|Diagnostic::new("WFT-ENGINE-NAVIGATION","input","Closed source Key identity required"))?;
                        let prepared=crate::application_navigation::prepare_sdk(&catalog,input,navigation_parameters,&selected,ordered.base())?;
                        let base=ordered.base_mut();for cap in &prepared.source().required_capabilities{if !base.required_capabilities.contains(cap){base.required_capabilities.push(cap.clone());}}
                        base.required_capabilities.push("relationship.sourceSelection".into());base.required_capabilities.sort();base.required_capabilities.dedup();base.navigation=Some(Box::new(prepared));
                    }
                    let artifact=self.emit(&req.interface_version,&req.dialect,&catalog,Plan::Ordered(&ordered),req.target,req.options.allow_candidate,factory)?;
                    let slots=ordered.sort_slots().iter().map(|s|{let(v,state)=s.hidden_outputs();json!({"field":s.identity(),"carrier":s.logical_type().family,"facets":s.logical_type().facets,"direction":if s.descending(){"desc"}else{"asc"},"nulls":if s.nulls_first(){"first"}else{"last"},"nullable":s.nullable(),"missing":s.missing(),"valueOutput":v,"stateOutput":state})}).collect::<Vec<_>>();
                    return Ok(json!({"interfaceVersion":"weft-sql-page/0.3","status":"compiled","artifact":artifact,"page":{"version":"weft.engine-cursor/0.2","pageSize":size,"visibleOutputs":ordered.base().outputs.len(),"selectedKey":selected,"key":ordered.selected_key(),"sortSlots":slots,"keySlots":ordered.key_slots(),"keyOutputs":ordered.key_slots().iter().map(|i|ordered.sort_slots()[*i].hidden_outputs().0).collect::<Vec<_>>()}}));
                }
                #[cfg(feature="engine-ordered-page-candidate")]
                if sdk_order_key.is_some(){query.order.clear();query.order_nulls.clear();}
                let sdk_read_parameters=req.parameters.clone().unwrap_or_default();
                let page_plan = if let Some((size, after, typed, decimal12)) = page {
                    if decimal12 && !matches!(req.interface_version.as_str(),"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8") { return Err(Diagnostic::new("WFT-PROFILE","input","Decimal12 page requires explicit compile0.2.6 context")); }
                    if (req.read_profile.is_some() && (req.interface_version!="weft-compile/0.2.8" || req.read_profile.as_ref().is_none_or(|p|p.version!="weft-truss-entity-read/0.4.0" || p.subset!=crate::application_ir::Subset::CompleteEntityPage))) || req.security.is_some() {
                        return Err(Diagnostic::new("WFT-PROFILE", "input", "Page entrypoint owns its selected read profile"));
                    }
                    Some(if let Some(profile)=req.read_profile.clone() {crate::application_resolve::resolve_entity_page_with_exact(&catalog,query.clone(),req.parameters.clone().unwrap_or_default(),req.exact_parameters.clone().unwrap_or_default(),size,after.as_deref(),typed,timestamp_equality,decimal12,profile)?} else if matches!(req.interface_version.as_str(),"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8") {crate::application_resolve::resolve_page_with_exact(&catalog,query.clone(),req.parameters.clone().unwrap_or_default(),req.exact_parameters.clone().unwrap_or_default(),size,after.as_deref(),typed,timestamp_equality,decimal12)?} else if typed { crate::application_resolve::resolve_typed_page(&catalog, query.clone(), req.parameters.clone().unwrap_or_default(), size, after.as_deref())? } else { crate::application_resolve::resolve_page(&catalog, query.clone(), req.parameters.clone().unwrap_or_default(), size, after.as_deref())? })
                } else { None };
                let mut plan = match page_plan.as_ref() {
                    Some(page) => page.plan.clone(),
                    None => if matches!(req.interface_version.as_str(),"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8") {crate::application_resolve::resolve_with_exact(&catalog,query,req.parameters.unwrap_or_default(),req.exact_parameters.unwrap_or_default(),req.read_profile,timestamp_equality)?} else {crate::application_resolve::resolve(&catalog, query, req.parameters.unwrap_or_default(), req.read_profile)?},
                };
                #[cfg(feature="engine-ordered-page-candidate")]
                if let Some(key)=sdk_order_key {
                 if !sdk_scalar_presence||ordered_page.is_some(){return Err(Diagnostic::new("WFT-ORDERED-READ","input","Explicit SDK ordered-read profile required"));}
                 let key=serde_json::from_value(key.clone()).map_err(|_|Diagnostic::new("WFT-ORDERED-READ","input","Closed original order Key required"))?;
                 plan=crate::application_ordered_page::prepare_read(&catalog,&req.sql,sdk_read_parameters,&key)?;
                }
                if cursor_delivery {
                    if !matches!(req.interface_version.as_str(),"weft-compile/0.2.7"|"weft-compile/0.2.8") || page_plan.is_none() {
                        return Err(Diagnostic::new("WFT-PROFILE","input","Cursor delivery requires an explicit authored-Key compile0.2.7 plan"));
                    }
                    // The logical plan describes the unlimited portal, not the
                    // bounded fetches performed by its separately admitted host.
                    plan.limit = None;
                }
                validate_application_plan(&catalog,&mut plan,&req.interface_version,timestamp_equality)?;
                if let Some(input)=navigation{
                    #[cfg(feature="engine-ordered-page-candidate")]
                    let mut prepared=if sdk_scalar_presence{let selected:crate::application_ordered_page::SelectedKey=serde_json::from_value(sdk_source_key.unwrap().clone()).map_err(|_|Diagnostic::new("WFT-ENGINE-NAVIGATION","input","Closed original source Key required"))?;crate::application_navigation::prepare_sdk(&catalog,input,navigation_parameters,&selected,&plan)?}else{crate::application_navigation::prepare(&catalog,input,navigation_parameters,navigation_exact,timestamp_equality,matches!(req.interface_version.as_str(),"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8"),&plan)?};
                    #[cfg(not(feature="engine-ordered-page-candidate"))]
                    let mut prepared=crate::application_navigation::prepare(&catalog,input,navigation_parameters,navigation_exact,timestamp_equality,matches!(req.interface_version.as_str(),"weft-compile/0.2.5"|"weft-compile/0.2.6"|"weft-compile/0.2.7"|"weft-compile/0.2.8"),&plan)?;
                    validate_application_plan(&catalog,prepared.source_mut(),&req.interface_version,timestamp_equality)?;
                    for cap in &prepared.source().required_capabilities{if !plan.required_capabilities.contains(cap){plan.required_capabilities.push(cap.clone());}}
                    plan.required_capabilities.push("relationship.sourceSelection".into());plan.required_capabilities.sort();plan.required_capabilities.dedup();
                    plan.ir_version=if sdk_scalar_presence{"weft-ir/engine-navigation/0.2"}else{"weft-ir/engine-navigation/0.1"}.into();plan.navigation=Some(Box::new(prepared));
                }
                let artifact = self.emit(
                    &req.interface_version, &req.dialect, &catalog, Plan::V02(&plan),
                    req.target, req.options.allow_candidate, factory,
                )?;
                if let Some(page) = page_plan {
                    if cursor_delivery {
                        return Ok(json!({"interfaceVersion":"weft-sql-cursor/0.1","status":"compiled","artifact":artifact,
                            "cursor":{"visibleOutputs":page.visible_outputs,"keyOutputs":page.key_outputs,"batchSize":page.page_size,
                                "generatedOrderPositions":(0..page.plan.order.len()).collect::<Vec<_>>()}}));
                    }
                    return Ok(json!({"interfaceVersion":if page.typed_keys {"weft-sql-page/0.2"} else {"weft-sql-page/0.1"}, "status":"compiled", "artifact":artifact,
                        "page":{"visibleOutputs":page.visible_outputs,"keyOutputs":page.key_outputs,"pageSize":page.page_size,
                            "generatedContinuationFilter":if page.continuation_values.is_empty(){None}else{Some(page.plan.filters.len()-1)},
                            "generatedOutputPositions":page.key_outputs,
                            "generatedOrderPositions":(0..page.plan.order.len()).collect::<Vec<_>>(),
                            "generatedSpans":"diagnostic-anchors"}}));
                }
                Ok(artifact)
            }
        })();
        let output = match result {
            Ok(v) => v,
            Err(d) => {
                json!({"interfaceVersion":version,"status":"blocked","diagnostics":[public_versioned_diagnostic(d, &version, diagnostic_sql.as_deref())]})
            }
        };
        // serde_json maps use Unicode scalar key order; arrays/value strings retain order/text.
        serde_json::to_string(&output).expect("serializable compiler response")
    }
    fn emit(
        &self,
        version: &str,
        dialect: &str,
        catalog: &crate::model::Catalog,
        plan: Plan<'_>,
        target: TargetRequest,
        allow_candidate: bool,
        factory: Option<&mut RegistryFactory<'_>>,
    ) -> Result<Value> {
        let composed = match factory {
            Some(factory) => Some(factory(
                catalog,
                plan,
                CompositionInput {
                    backend_id: &target.backend_id,
                    backend_version: &target.backend_version,
                    target_profile: &target.target_profile,
                    binding_json: &target.binding_json,
                    binding_sha256: &target.binding_sha256,
                    allow_candidate,
                },
            )?),
            None => None,
        };
        let registry = composed.as_ref().unwrap_or(&self.registry);
        let manifest = registry.manifest(&target.backend_id).ok_or_else(|| {
            Diagnostic::new(
                "WFT-BACKEND-MISSING",
                "capability",
                "Selected backend is not registered",
            )
        })?;
        let binding = BindingInput {
            profile: manifest.binding_profile.clone(),
            json: target.binding_json,
            sha256: target.binding_sha256,
        };
        let target = Target {
            backend_id: target.backend_id,
            backend_version: target.backend_version,
            profile_id: target.target_profile,
            allow_candidate,
        };
        let compiled = registry.compile(catalog, plan, &target, &binding)?;
        let candidate = compiled
            .qualifications
            .iter()
            .any(|q| q.assessment.status == Status::Candidate);
        let evidence: std::collections::BTreeSet<_> = compiled
            .qualifications
            .iter()
            .flat_map(|q| q.assessment.evidence.iter().cloned())
            .collect();
        let logical_plan = match plan {
            Plan::V01(p) => serde_json::to_value(p),
            Plan::V02(p) => serde_json::to_value(p),
            Plan::V03(p) => serde_json::to_value(p),
            #[cfg(feature="engine-ordered-page-candidate")]
            Plan::Ordered(p)=>serde_json::to_value(p),
        }
        .expect("typed plan");
        let columns = if version == "weft-compile/0.1.0" {
            let crate::ir::Node::Project { outputs, .. } = &match plan {
                Plan::V01(p) => p,
                Plan::V02(_) | Plan::V03(_) => unreachable!(),
                #[cfg(feature="engine-ordered-page-candidate")]
                Plan::Ordered(_)=>unreachable!(),
            }
            .root
            else {
                unreachable!()
            };
            compiled.emission.columns.iter().zip(outputs).map(|(c,o)|{
    let Representation::Scalar{logical_type,carrier,decoder}=&c.representation else{unreachable!()};
    json!({"position":c.position,"outputName":c.output_name,"logicalType":logical_type,"nullable":c.nullable,"carrier":carrier,"decoder":decoder,"sourceIdentities":c.source_identities,"aggregate":if matches!(o.expression,crate::ir::Expression::Sum{..}){Some("sum")}else{None}})
   }).collect::<Vec<_>>()
        } else {
            compiled
                .emission
                .columns
                .iter()
                .map(|c| serde_json::to_value(c).expect("typed column"))
                .collect()
        };
        Ok(
            json!({"interfaceVersion":version,"status":"compiled","diagnostics":[],"dialect":dialect,"compilerVersion":concat!("weft/",env!("CARGO_PKG_VERSION")),"modelPins":catalog.pins(),
   "backend":{"backendId":compiled.backend_id,"backendVersion":compiled.backend_version,"targetProfile":compiled.target_profile.id,"interfaceVersion":manifest.interface_version},
   "targetContext":compiled.target_profile,"bindingSha256":compiled.binding_sha256,"logicalPlan":logical_plan,"sql":compiled.emission.sql,"parameters":compiled.emission.parameters,"columns":columns,"obligations":compiled.emission.obligations,
   "qualification":{"status":if candidate{"candidate"}else{"conformance-verified"},"evidence":evidence,"assumptions":[],"operations":compiled.qualifications}}),
        )
    }
}
#[path = "compile04.rs"]
pub mod v04;

fn public_versioned_diagnostic(mut d: Diagnostic, version: &str, sql: Option<&str>) -> Value {
    if version=="weft-compile/0.2.8" {
        if let Some(reason)=d.resolution_reason {d.code=reason.code().into();}
        if d.code=="WFT-UNSUPPORTED" && d.phase=="parse" {
            if let Some(sql)=sql {
                // Reuse the existing owner syntax recognizer only to classify
                // refusal. Its AST never bypasses the admitted typed frontend.
                match sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect{},sql) {
                    Ok(_)=>{},
                    Err(sqlparser::parser::ParserError::RecursionLimitExceeded)=>d.code="WFT-LIMIT".into(),
                    Err(_)=>d.code="WFT-SYNTAX".into(),
                }
            }
        }
    }
    public_diagnostic(d)
}

#[cfg(test)]
mod selective_diagnostic_tests {
    use super::*;
    #[test]
    fn resolution_categories_are_versioned_and_do_not_infer_source_existence() {
        let cases:Vec<Value>=serde_json::from_str(include_str!("../../../tests/truss-entity-selection/cases.json")).unwrap();
        let mut request=cases[0]["request"].clone();request.as_object_mut().unwrap().remove("readProfile");
        let compiler=Compiler::default();
        for (sql,new_code,old_code) in [
            ("SELECT a.missing FROM \"A\" a","WFT-COLUMN-MISSING","WFT-NAME-MISSING"),
            ("SELECT a.* FROM \"unsupplied\" a","WFT-SOURCE-UNAVAILABLE","WFT-NAME-MISSING"),
            ("SELECT absent.\"A-label\" FROM \"A\" a","WFT-SOURCE-UNAVAILABLE","WFT-NAME-MISSING"),
        ] {
            request["sql"]=json!(sql);
            for (version,code) in [("weft-compile/0.2.7",old_code),("weft-compile/0.2.8",new_code)] {
                request["interfaceVersion"]=json!(version);
                let mut factory=|_: &crate::model::Catalog,_: Plan<'_>,_: CompositionInput<'_>|Ok(Registry::default());
                let result:Value=serde_json::from_str(&compiler.compile_identity_names_json_with_exact_numbers_factory(&request.to_string(),&mut factory)).unwrap();
                assert_eq!(result["status"],"blocked","{result}");
                assert_eq!(result["diagnostics"][0]["code"],code,"{sql}: {result}");
                assert!(result["diagnostics"][0].get("resolutionReason").is_none());
            }
        }
        // UMF source names may be ambiguous even though a native consumer can
        // refuse such a catalog. Resolve this supplied original before binding.
        let mut document:Value=serde_json::from_str(request["modules"][0]["documentJson"].as_str().unwrap()).unwrap();
        for element in document["modules"][0]["elements"].as_array_mut().unwrap() {
            if element["id"]=="A-label" || element["id"]=="A-value" {element["name"]=json!("collision");}
        }
        let original=document.to_string();request["modules"][0]["documentJson"]=json!(original);
        request["modules"][0]["pin"]["sha256"]=json!(crate::json::sha256(original.as_bytes()));
        request["sql"]=json!("SELECT a.collision FROM \"A\" a");
        for (version,code) in [("weft-compile/0.2.7","WFT-NAME-AMBIGUOUS"),("weft-compile/0.2.8","WFT-COLUMN-AMBIGUOUS")] {
            request["interfaceVersion"]=json!(version);
            let mut factory=|_: &crate::model::Catalog,_: Plan<'_>,_: CompositionInput<'_>|Ok(Registry::default());
            let result:Value=serde_json::from_str(&compiler.compile_identity_names_json_with_exact_numbers_factory(&request.to_string(),&mut factory)).unwrap();
            assert_eq!(result["diagnostics"][0]["code"],code,"{result}");
        }
    }
}

fn public_diagnostic(d: Diagnostic) -> Value {
    let recovery = match d.code.as_str() {
        "WFT-BACKEND-MISSING" | "WFT-BACKEND-VERSION" | "WFT-CAPABILITY" => "change-profile",
        "WFT-BACKEND-FAILURE" | "WFT-EMIT" | "WFT-OBLIGATION" => "host-action",
        _ => "correct-input",
    };
    let mut value = serde_json::to_value(d).expect("typed diagnostic");
    value["recoverability"] = json!(recovery);
    value
}

#[path = "compile041.rs"]
pub mod v041;

#[path = "compile042.rs"]
pub mod v042;

fn validate_application_plan(catalog:&crate::model::Catalog,plan:&mut crate::application_ir::Plan,interface_version:&str,timestamp_equality:bool)->Result<()>{
                if interface_version=="weft-compile/0.2.8" {
                    plan.ir_version="weft-ir/0.2.8".into();
                    if !Plan028::is_valid(&serde_json::to_value(&*plan).expect("typed selective query plan")) {return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates selective query semantics"));}
                } else if catalog.exact_dates {
                    plan.ir_version="weft-ir/0.2.7".into();
                    if !Plan027::is_valid(&serde_json::to_value(&*plan).expect("typed canonical date plan")) {return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates canonical date semantics"));}
                } else if timestamp_equality {
                    plan.ir_version="weft-ir/0.2.6".into();
                    if !Plan026::is_valid(&serde_json::to_value(&*plan).expect("typed exact instant plan")) {return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates exact instant semantics"));}
                } else if interface_version=="weft-compile/0.2.5" {
                    plan.ir_version="weft-ir/0.2.5".into();
                    if !Plan025::is_valid(&serde_json::to_value(&*plan).expect("typed exact equality plan")) {return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates exact equality semantics"));}
                } else if interface_version == "weft-compile/0.2.4" {
                    plan.ir_version="weft-ir/0.2.4".into();
                    if !Plan024::is_valid(&serde_json::to_value(&*plan).expect("typed import entity plan")) { return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates import entity semantics")); }
                } else if interface_version == "weft-compile/0.2.3" {
                    plan.ir_version="weft-ir/0.2.3".into();
                    if !Plan023::is_valid(&serde_json::to_value(&*plan).expect("typed entity plan")) { return Err(Diagnostic::new("WFT-IR","resolve","Resolved plan violates timestamp entity semantics")); }
                } else if interface_version == "weft-compile/0.2.2" {
                    plan.ir_version = "weft-ir/0.2.2".into();
                    let value = serde_json::to_value(&*plan).map_err(|_| Diagnostic::new("WFT-IR", "resolve", "Invalid resolved entity plan"))?;
                    if !Plan022::is_valid(&value) { return Err(Diagnostic::new("WFT-IR", "resolve", "Resolved plan violates versioned entity semantics")); }
                } else if catalog.exact_scalars { plan.ir_version = "weft-ir/0.2.1".into(); }

    Ok(())
}
