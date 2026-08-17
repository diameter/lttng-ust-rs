use bindgen::Builder;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs::File, path::Path};

use super::ctf_field_c_type;
use crate::{EventClass, EventInstance, Field, Provider};

pub(super) fn generate_interface_impl(
    path: &PathBuf,
    providers: &[Provider],
    interface_header: &Path,
    tracepoint_header: &Path,
) -> io::Result<()> {
    let mut outf = File::create(path).unwrap_or_else(|_| {
        panic!(
            "Failed to create tracepoint interface implementation {:?}\n",
            path
        )
    });

    writeln!(outf, "#include \"{}\"", interface_header.to_string_lossy())?;
    //writeln!(outf, "#include \"{}\"", tracepoint_header.to_string_lossy())?;

    for provider in providers {
        generate_provider_impl(provider, &mut outf)?;
    }

    Ok(())
}

pub(super) fn generate_interface_header(path: &PathBuf, providers: &[Provider]) -> io::Result<()> {
    let mut outf = File::create(path)
        .unwrap_or_else(|_| panic!("Failed to create tracepoint interface header {:?}\n", path));

    writeln!(outf, "#if !defined(_RUST_TRACEPOINT_INTERFACE)")?;
    writeln!(outf, "#define _RUST_TRACEPOINT_INTERFACE")?;
    writeln!(outf, "#include <stdint.h>")?;
    writeln!(outf, "#include <stddef.h>")?;

    for provider in providers {
        generate_provider_header(provider, &mut outf)?;
    }

    write!(outf, "#endif")?;
    Ok(())
}

pub(super) fn whitelist_interface(providers: &[Provider], mut b: Builder) -> Builder {
    for provider in providers {
        for event_class in &provider.classes {
            for instance in &event_class.instances {
                let fname = generate_func_name(provider, event_class, instance);
                eprintln!("whitelisting: {}", fname);
                b = b.allowlist_function(fname);
            }
        }
    }
    b
}

fn generate_provider_impl<F: Write>(provider: &Provider, outf: &mut F) -> io::Result<()> {
    for event_class in &provider.classes {
        for instance in &event_class.instances {
            write!(
                outf,
                "void {}(",
                generate_func_name(provider, event_class, instance)
            )?;
            generate_c_args(&event_class.fields, outf, true)?;
            writeln!(outf, ") {{")?;
            write!(
                outf,
                "    printf(\"en=%ld\", lttng_ust_tracepoint_enabled({}, {}));",
                provider.name, instance.name
            )?;
            write!(
                outf,
                "    lttng_ust_tracepoint({}, {}, ",
                provider.name, instance.name
            )?;
            generate_c_args(&event_class.fields, outf, false)?;
            writeln!(outf, ");")?;
            write!(outf, "}}\n\n")?;
        }
    }

    Ok(())
}

fn generate_provider_header<F: Write>(provider: &Provider, outf: &mut F) -> io::Result<()> {
    for event_class in &provider.classes {
        for instance in &event_class.instances {
            write!(
                outf,
                "extern void {}(",
                generate_func_name(provider, event_class, instance)
            )?;
            generate_c_args(&event_class.fields, outf, true)?;
            writeln!(outf, ");")?;
        }
    }

    Ok(())
}

pub fn generate_func_name(
    provider: &Provider,
    event_class: &EventClass,
    instance: &EventInstance,
) -> String {
    format!(
        "{}_{}_{}_tp",
        provider.name, event_class.class_name, instance.name
    )
}

fn generate_c_args<F: Write>(fields: &[Field], outf: &mut F, include_type: bool) -> io::Result<()> {
    let mut first = true;
    for field in fields {
        if first {
            first = false
        } else {
            write!(outf, ", ")?;
        }
        if include_type {
            write!(
                outf,
                "{} {}_arg",
                ctf_field_c_type(field.ctf_type),
                field.name
            )?;
            if field.ctf_type.is_sequence() {
                write!(outf, ", size_t {}_len", field.name)?;
            }
        } else {
            write!(outf, "{}_arg", field.name)?;
            if field.ctf_type.is_sequence() {
                write!(outf, ", {}_len", field.name)?;
            }
        }
    }
    Ok(())
}
