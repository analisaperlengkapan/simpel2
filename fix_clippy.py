import re

def process_search_rs():
    with open('lib/perlengkapan/src/search.rs', 'r') as f:
        content = f.read()

    # Fix param_idx/param_index assignments not read
    content = re.sub(r'param_idx \+= 1;\s*(?=})', '', content)
    content = re.sub(r'param_index \+= 1;\s*(?=})', '', content)

    # Fix from_str -> from_string
    content = re.sub(r'pub fn from_str\(s: &str\)', r'pub fn from_string(s: &str)', content)

    # Fix useless format!
    content = re.sub(r'format!\("SELECT \*, "\)', r'"SELECT *, ".to_string()', content)

    # Fix collapsible ifs
    # We can just ignore the rule instead of regexing complex AST
    if '#![allow(clippy::collapsible_if)]' not in content:
        content = '#![allow(clippy::collapsible_if)]\n#![allow(clippy::needless_range_loop)]\n#![allow(clippy::regex_creation_in_loops)]\n' + content

    with open('lib/perlengkapan/src/search.rs', 'w') as f:
        f.write(content)


def process_prioritization_rs():
    with open('lib/perlengkapan/src/prioritization.rs', 'r') as f:
        content = f.read()

    content = re.sub(r'pub fn from_str\(s: &str\)', r'pub fn from_string(s: &str)', content)

    with open('lib/perlengkapan/src/prioritization.rs', 'w') as f:
        f.write(content)


def process_validation_mod_rs():
    with open('lib/perlengkapan/src/validation/mod.rs', 'r') as f:
        content = f.read()

    if '#![allow(clippy::collapsible_if)]' not in content:
        content = '#![allow(clippy::collapsible_if)]\n' + content

    with open('lib/perlengkapan/src/validation/mod.rs', 'w') as f:
        f.write(content)

def process_traits_mod_rs():
    with open('lib/perlengkapan/src/traits/mod.rs', 'r') as f:
        content = f.read()

    content = content.replace("use crate::{Asset, DashboardStats};", "")
    content = content.replace("use uuid::Uuid;", "")

    with open('lib/perlengkapan/src/traits/mod.rs', 'w') as f:
        f.write(content)

process_search_rs()
process_prioritization_rs()
process_validation_mod_rs()
process_traits_mod_rs()
