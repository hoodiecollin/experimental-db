use std::collections::HashMap;

use syn::visit::{self, Visit};

use super::{ColumnInfo, DataKind, FloatKind, IntegerKind, RelationInfo, RelationKind, TableInfo};

#[derive(Debug, Clone)]
pub struct TableInfoBuilder<'a> {
    pub other_tables: &'a [String],
    pub name: String,
    pub columns: HashMap<String, ColumnInfo>,
    pub references: Vec<RelationInfo>,
}

impl TableInfoBuilder<'_> {
    pub fn build(self) -> TableInfo {
        TableInfo {
            name: self.name,
            columns: self.columns,
            references: self.references,
        }
    }
}

impl<'ast> Visit<'ast> for TableInfoBuilder<'ast> {
    fn visit_field(&mut self, field: &'ast syn::Field) {
        let name = field.ident.as_ref().unwrap().to_string();
        let ty = &field.ty;

        match ty {
            syn::Type::Path(ty) => {
                if ty.path.segments.len() == 1 {
                    let segment = ty.path.segments.last().unwrap();
                    let ident = &segment.ident;

                    if handle_simple_types(&mut self.columns, &name, ident, true) {
                        return visit::visit_field(self, field);
                    }

                    match ident.to_string().as_str() {
                        "Option" => {
                            if let Some(inner_ident) = get_generic_arg(&segment.arguments) {
                                let inner_ident_name = inner_ident.to_string();

                                if handle_simple_types(&mut self.columns, &name, inner_ident, false)
                                {
                                    return visit::visit_field(self, field);
                                } else if &inner_ident_name == "Vec" {
                                    panic!("Arrays should not be optional: {:?}", field);
                                } else {
                                    if !self.other_tables.contains(&inner_ident_name) {
                                        panic!(
                                            "Table {:?} references unknown table: {:?}",
                                            self.name, inner_ident_name
                                        );
                                    }

                                    self.references.push(RelationInfo {
                                        required: false,
                                        property: name,
                                        target: inner_ident_name,
                                        kind: RelationKind::HasOne,
                                    });
                                }
                            } else {
                                panic!("Option type is missing a generic argument: {:?}", field);
                            }
                        }
                        "Vec" => {
                            if let Some(inner_ident) = get_generic_arg(&segment.arguments) {
                                let inner_ident_name = inner_ident.to_string();
                                let mut simple_kind = None;

                                match_simple_types(ident, |kind| {
                                    simple_kind = Some(kind);
                                });

                                if let Some(kind) = simple_kind {
                                    self.columns.insert(
                                        name,
                                        ColumnInfo::new(true, DataKind::Array(Box::new(kind))),
                                    );
                                } else {
                                    if !self.other_tables.contains(&inner_ident_name) {
                                        panic!(
                                            "Table {:?} references unknown table: {:?}",
                                            self.name, inner_ident_name
                                        );
                                    }

                                    self.references.push(RelationInfo {
                                        required: false,
                                        property: name,
                                        target: inner_ident_name,
                                        kind: RelationKind::HasMany,
                                    });
                                }
                            } else {
                                panic!("Vec type is missing a generic argument: {:?}", field);
                            }
                        }
                        _ => {
                            let ident_name = ident.to_string();

                            if !self.other_tables.contains(&ident_name) {
                                panic!(
                                    "Table {:?} references unknown table: {:?}",
                                    self.name, ident_name
                                );
                            }

                            self.references.push(RelationInfo {
                                required: true,
                                property: name,
                                target: ident_name,
                                kind: RelationKind::HasOne,
                            });
                        }
                    }
                } else {
                    panic!("Imported types are not supported: {:?}", ty);
                }
            }
            _ => {
                panic!("Unsupported type: {:?}", ty);
            }
        }

        fn get_generic_arg(arguments: &syn::PathArguments) -> Option<&syn::Ident> {
            match arguments {
                syn::PathArguments::AngleBracketed(ref args) => {
                    if args.args.len() == 1 {
                        if let syn::GenericArgument::Type(ref ty) = args.args.first().unwrap() {
                            if let syn::Type::Path(ref ty) = ty {
                                if ty.path.segments.len() == 1 {
                                    let segment = ty.path.segments.last().unwrap();
                                    return Some(&segment.ident);
                                }
                            }
                        }
                    }
                }
                _ => {}
            };

            None
        }

        fn handle_simple_types(
            cols: &mut HashMap<String, ColumnInfo>,
            name: &str,
            ident: &syn::Ident,
            required: bool,
        ) -> bool {
            match_simple_types(ident, |kind| {
                cols.insert(name.to_owned(), ColumnInfo::new(required, kind));
            })
        }

        fn match_simple_types(ident: &syn::Ident, mut f: impl FnMut(DataKind)) -> bool {
            match ident.to_string().as_str() {
                "String" => f(DataKind::String),
                "bool" => f(DataKind::Boolean),
                "f32" => f(DataKind::Float(FloatKind::F32)),
                "f64" => f(DataKind::Float(FloatKind::F64)),
                "i8" => f(DataKind::Integer(IntegerKind::I8)),
                "i16" => f(DataKind::Integer(IntegerKind::I16)),
                "i32" => f(DataKind::Integer(IntegerKind::I32)),
                "i64" => f(DataKind::Integer(IntegerKind::I64)),
                "isize" => f(DataKind::Integer(IntegerKind::ISize)),
                "i128" => f(DataKind::Integer(IntegerKind::I128)),
                "u8" => f(DataKind::Integer(IntegerKind::U8)),
                "u16" => f(DataKind::Integer(IntegerKind::U16)),
                "u32" => f(DataKind::Integer(IntegerKind::U32)),
                "u64" => f(DataKind::Integer(IntegerKind::U64)),
                "usize" => f(DataKind::Integer(IntegerKind::USize)),
                "u128" => f(DataKind::Integer(IntegerKind::U128)),
                _ => return false,
            };

            true
        }
    }
}
