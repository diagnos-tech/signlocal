//! SPEC §6.3: ICP-Brasil detection, level, holder, CPF and CNPJ.
//!
//! Certificate values come from `tests/fixtures/gen/certs-icp.sh`:
//! CPF 12345678901 (birth 15051990) for the person, CNPJ 12345678000195 for
//! the company, CPF 98765432100 for the company's responsible person and
//! CPF 22222222222 as a second value to tell the otherNames apart.

mod cnpj;
mod cpf;
mod detection;
mod formatting;
mod holder_name;
mod level;
mod reference;
mod types;

#[path = "../common/mod.rs"]
mod common;

use std::collections::HashSet;

use common::info;
use websign_core::{IcpBrasil, IcpLevel};

use IcpLevel::{A1, A2, A3, A4, Other, S1, S2, S3, S4, T3, T4};

const CPF: &str = "12345678901";
const CNPJ: &str = "12345678000195";
const RESPONSIBLE_CPF: &str = "98765432100";
const OTHER_CPF: &str = "22222222222";

fn icp(name: &str) -> IcpBrasil {
    info(name)
        .icp_brasil
        .unwrap_or_else(|| panic!("{name} should be recognized as ICP-Brasil"))
}

fn expected(
    level: Option<IcpLevel>,
    holder: Option<&str>,
    cpf: Option<&str>,
    cnpj: Option<&str>,
) -> IcpBrasil {
    IcpBrasil {
        level,
        holder_name: holder.map(str::to_owned),
        cpf: cpf.map(str::to_owned),
        cnpj: cnpj.map(str::to_owned),
    }
}

fn person(cpf: Option<&str>) -> IcpBrasil {
    expected(Some(A3), Some("ANA BEATRIZ SOUZA"), cpf, None)
}

fn company(cnpj: Option<&str>, cpf: Option<&str>) -> IcpBrasil {
    expected(Some(A1), Some("EMPRESA TESTE LTDA"), cpf, cnpj)
}
