use crate::infer_schema::{
    ColumnInfo, DataKind, FloatKind, IntegerKind, RelationInfo, RelationKind, Schema, TableInfo,
};

use inflection_rs::inflection::{pluralize, titleize, underscore};
use quote::{format_ident, ToTokens};

impl TableInfo {
    pub fn model_struct_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.name)
    }

    pub fn model_builder_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Builder", self.name)
    }

    pub fn table_struct_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Table", self.name)
    }

    pub fn property_enum_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}Properties", self.name)
    }

    pub fn app_property_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", pluralize(underscore(self.name.clone())))
    }
}

impl RelationInfo {
    pub fn target_model_ident(&self) -> proc_macro2::Ident {
        format_ident!("{}", self.target)
    }

    // pub fn target_properties_enum_ident(&self) -> proc_macro2::Ident {
    //     format_ident!("{}Properties", self.target)
    // }
}

pub fn render_table_items(schema: &Schema) -> Vec<proc_macro2::TokenStream> {
    let mut all_tables = schema.tables.iter().collect::<Vec<_>>();

    let mut table_enum_variants = vec![];

    let mut app_property_defs = vec![];
    let mut app_property_constructors = vec![];

    let mut core_items = vec![quote::quote! {
        pub type AnyResult<T> = Result<T, Box<dyn std::error::Error>>;
    }];

    all_tables.sort_by(|(a, _), (b, _)| a.cmp(b));

    for (_, table_info) in all_tables {
        let mut variant_idents = vec![];
        let mut variants = vec![];

        let mut property_idents = vec![];
        let mut property_defs = vec![];
        let mut optional_property_defs = vec![];

        let mut table_column_defs = vec![];

        let mut model_property_constructors = vec![];
        let mut model_property_setters = vec![];
        // let mut model_property_getters = vec![];

        let mut optional_model_property_constructors = vec![];
        let mut optional_model_property_setters = vec![];
        let mut optional_model_property_erasers = vec![];
        // let mut optional_model_property_getters = vec![];

        let model_struct_ident = table_info.model_struct_ident();
        let model_builder_ident = table_info.model_builder_ident();
        let table_struct_ident = table_info.table_struct_ident();
        let property_enum_ident = table_info.property_enum_ident();
        let app_property_ident = table_info.app_property_ident();

        table_enum_variants.push(quote::quote! {
            #table_struct_ident {
                property_id: #property_enum_ident,
                entity_id: engine::Id,
            }
        });

        app_property_defs.push(quote::quote! {
            pub #app_property_ident: #table_struct_ident
        });

        app_property_constructors.push(quote::quote! {
            #app_property_ident: #table_struct_ident::new()
        });

        for (property, info) in &table_info.columns {
            let variant_ident = format_ident!("{}", titleize(property));
            let property_ident = format_ident!("{}", property);
            let data_type = &info.kind;

            variant_idents.push(variant_ident.clone());

            variants.push(quote::quote! {
                #variant_ident
            });

            property_idents.push(property_ident.clone());

            table_column_defs.push(quote::quote! {
                pub #property_ident: engine::Column<#data_type>
            });

            if info.required {
                property_defs.push(quote::quote! {
                    #[builder(default)]
                    pub #property_ident: #data_type
                });

                model_property_constructors.push(quote::quote! {
                    #property_ident: Default::default()
                });

                model_property_setters.push(quote::quote! {
                    self.#property_ident = value;
                });
            } else {
                optional_property_defs.push(quote::quote! {
                    #[builder(default, setter(strip_option))]
                    pub #property_ident: Option<#data_type>
                });

                optional_model_property_constructors.push(quote::quote! {
                    #property_ident: Default::default()
                });

                optional_model_property_setters.push(quote::quote! {
                    self.#property_ident = Some(value);
                });

                optional_model_property_erasers.push(quote::quote! {
                    self.#property_ident = None;
                });
            }
        }

        for reference in &table_info.references {
            let variant_ident = format_ident!("{}", titleize(&reference.property));
            let property_ident = format_ident!("{}", reference.property);

            let target_model_ident = reference.target_model_ident();
            // let target_properties = reference.target_properties_enum_ident();

            variant_idents.push(variant_ident.clone());

            // this isn't right
            variants.push(quote::quote! {
                #variant_ident
            });

            property_idents.push(property_ident.clone());

            table_column_defs.push(quote::quote! {
                pub #property_ident: engine::Column<engine::Id>
            });

            match reference.kind {
                RelationKind::HasOne => {
                    if reference.required {
                        property_defs.push(quote::quote! {
                            #[builder(default)]
                            pub #property_ident: #target_model_ident
                        });

                        model_property_constructors.push(quote::quote! {
                            #property_ident: Default::default()
                        });

                        model_property_setters.push(quote::quote! {
                            self.#property_ident = value;
                        });
                    } else {
                        optional_property_defs.push(quote::quote! {
                            #[builder(default, setter(strip_option))]
                            pub #property_ident: Option<#target_model_ident>
                        });

                        optional_model_property_constructors.push(quote::quote! {
                            #property_ident: Default::default()
                        });

                        optional_model_property_setters.push(quote::quote! {
                            self.#property_ident = Some(value);
                        });

                        optional_model_property_erasers.push(quote::quote! {
                            self.#property_ident = None;
                        });
                    }
                }
                RelationKind::HasMany => {
                    if reference.required {
                        property_defs.push(quote::quote! {
                            #[builder(default)]
                            pub #property_ident: Vec<#target_model_ident>
                        });

                        model_property_constructors.push(quote::quote! {
                            #property_ident: Default::default()
                        });

                        model_property_setters.push(quote::quote! {
                            self.#property_ident = value;
                        });
                    } else {
                        optional_property_defs.push(quote::quote! {
                            #[builder(default, setter(strip_option))]
                            pub #property_ident: Option<Vec<#target_model_ident>>
                        });

                        optional_model_property_constructors.push(quote::quote! {
                            #property_ident: Default::default()
                        });

                        optional_model_property_setters.push(quote::quote! {
                            self.#property_ident = Some(value);
                        });

                        optional_model_property_erasers.push(quote::quote! {
                            self.#property_ident = None;
                        });
                    }
                }
            }
        }

        core_items.push(quote::quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub enum #property_enum_ident {
                #(#variants,)*
            }

            #[derive(Debug)]
            pub struct #table_struct_ident {
                #(#table_column_defs,)*
            }

            impl #table_struct_ident {
                pub fn new() -> Self {
                    Self {
                        #(#property_idents: engine::Column::new(),)*
                    }
                }
            }

            #[derive(Builder, Debug, Default, Clone)]
            #[builder(setter(into))]
            pub struct #model_struct_ident {
                #[builder(default)]
                pub id: engine::Id,
                #(#property_defs,)*
                #(#optional_property_defs,)*
            }

            impl #model_struct_ident {
                pub fn new() -> Self {
                    Self {
                        id: engine::new_id(),
                        #(#model_property_constructors,)*
                        #(#optional_model_property_constructors,)*
                    }
                }

                pub fn builder() -> #model_builder_ident {
                    #model_builder_ident::default()
                }
            }
        });
    }

    core_items.push(quote::quote! {
        pub struct App {
            #(#app_property_defs,)*
        }

        impl App {
            pub fn new() -> Self {
                Self {
                    #(#app_property_constructors,)*
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum DataId {
            #(#table_enum_variants,)*
        }

        pub struct MutationQueue {
            pub file: std::fs::File,
            pub changes: hashbrown::HashMap<DataId, engine::DataValue>,
        }
    });

    core_items
}

impl ToTokens for ColumnInfo {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let kind = &self.kind;
        let required = self.required;

        if required {
            quote::quote!(#kind).to_tokens(tokens);
        } else {
            quote::quote!(Option<#kind>).to_tokens(tokens);
        }
    }
}

impl ToTokens for DataKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DataKind::String => quote::quote!(String).to_tokens(tokens),
            DataKind::Boolean => quote::quote!(bool).to_tokens(tokens),
            DataKind::Integer(kind) => kind.to_tokens(tokens),
            DataKind::Float(kind) => kind.to_tokens(tokens),
            DataKind::Array(kind) => {
                let kind = kind.as_ref();
                quote::quote!(Vec<#kind>).to_tokens(tokens)
            }
        }
    }
}

impl ToTokens for IntegerKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        format_ident!("{}", self.to_string()).to_tokens(tokens)
    }
}

impl ToTokens for FloatKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        format_ident!("{}", self.to_string()).to_tokens(tokens)
    }
}
