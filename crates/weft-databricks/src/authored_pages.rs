// SPDX-License-Identifier: MIT OR Apache-2.0
//! Separate candidate language registrations for original authored-Key pages.
//! Historical engine profiles remain unchanged; native publication and complete
//! key-integrity handling are still mandatory before any result is released.
use weft_core::{backend::*,error::Result};
pub const LOCAL_VERSION:&str="0.1.0-local-spark4-pages-candidate";
pub const LOCAL42_VERSION:&str="0.2.0-local-spark4.2-pages-candidate";
pub const MANAGED_VERSION:&str="0.1.0-managed-20261016-pages-candidate";
pub const MANAGED_CURRENT_VERSION:&str="0.1.0-managed-current-core-pages-candidate";
pub struct AuthoredPages<B>{backend:B,version:&'static str}
pub fn local()->AuthoredPages<crate::local_spark4::LocalSpark4>{AuthoredPages{backend:crate::local_spark4::LocalSpark4,version:LOCAL_VERSION}}
pub fn local42()->AuthoredPages<crate::local_spark42::LocalSpark42>{AuthoredPages{backend:crate::local_spark42::LocalSpark42,version:LOCAL42_VERSION}}
pub fn managed()->AuthoredPages<crate::managed_20261016::Managed20261016>{AuthoredPages{backend:crate::managed_20261016::Managed20261016,version:MANAGED_VERSION}}
pub fn managed_current_core()->AuthoredPages<crate::managed_current_pages::ManagedCurrentPages>{AuthoredPages{backend:crate::managed_current_pages::ManagedCurrentPages,version:MANAGED_CURRENT_VERSION}}
impl<B:Backend> Backend for AuthoredPages<B>{
 type Mapping=B::Mapping;type TargetPlan=B::TargetPlan;
 fn describe(&self)->Result<Manifest>{let mut manifest=self.backend.describe()?;manifest.backend_version=self.version.into();manifest.interface_version="weft-backend/0.2.7".into();
  let language=LanguageProfile{dialect_profile:"weft-sql/0.2.0".into(),ir_version:"weft-ir/0.2.7".into()};manifest.language_profiles.push(language.clone());
  for capability in &mut manifest.capabilities{capability.language_profiles.push(language.clone());capability.status=Status::Candidate;capability.evidence.clear();capability.constraints.push("Separate original authored-Key page language candidate; complete native publication and key integrity required".into());}
  manifest.evidence.clear();Ok(manifest)
 }
 fn validate_binding(&self,c:&Context<'_>)->Result<Validated<Self::Mapping>>{self.backend.validate_binding(c)}
 fn assess(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Vec<Assessment>>{self.backend.assess(c,m)}
 fn lower(&self,c:&Context<'_>,m:&Self::Mapping)->Result<Self::TargetPlan>{self.backend.lower(c,m)}
 fn emit(&self,c:&Context<'_>,p:&Self::TargetPlan)->Result<Emission>{self.backend.emit(c,p)}
}
