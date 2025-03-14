use proc_macro::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, *};

#[proc_macro_derive(PointerDerefDebugPrint)]
pub fn pointer_deref_debug_print(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Extract generics for implementing the trait with the same generics
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    // Create a new where clause with additional bounds
    let mut new_where_clause = where_clause.cloned().unwrap_or_else(|| 
        WhereClause {
            where_token: parse_quote!(where),
            predicates: Punctuated::new(),
        }
    );
    
    // Add bounds for each type parameter
    for param in &generics.params {
        if let GenericParam::Type(type_param) = param {
            let param_ident = &type_param.ident;
            // Add 'static bound for all type parameters
            new_where_clause.predicates.push(parse_quote!(
                #param_ident: ::std::fmt::Debug + 'static
            ));
        }
    }
    
    // Generate field debug code by iterating through fields
    let field_debugs = if let Data::Struct(data) = &input.data {
        data.fields.iter().map(|field| {
            let field_name = &field.ident;
            let field_name_str = field_name.as_ref().unwrap().to_string();
            
            if field_name_str.contains("_pad") {
                // Don't print padding fields
                return quote! {}; // Empty quote
            }
            
            // Check if this field is a pointer type
            let is_pointer = match &field.ty {
                Type::Path(TypePath { path, .. }) => path.segments.iter().any(|seg| {
                    let ident_str = seg.ident.to_string();
                    ident_str.contains("Pointer")
                }),
                _ => false,
            };
            
            // Extract the field type for better display
            let field_type = match &field.ty {
                Type::Path(TypePath { path, .. }) => {
                    if let Some(segment) = path.segments.last() {
                        let type_name = segment.ident.to_string();
                        quote! { #type_name }
                    } else {
                        quote! { "Unknown" }
                    }
                },
                _ => quote! { "Unknown" },
            };
            
            if is_pointer {
                quote! {
                    // Get the address to check if we've seen it before
                    let address = self.#field_name.address().to_umem();
                    
                    //println!("{}  {}: {} Pointer @ {:#x}", indent, #field_name_str, #field_type, address);
                    
                    // Only process this pointer if we haven't seen it before
                    if !visited_addresses.contains(&address) {
                        // Add this address to our visited set
                        visited_addresses.insert(address);
                        
                        // Read the pointer value using the memory view
                        match self.#field_name.read(mem) {
                            Ok(value) => {
                                // Get the type name if possible through any means available
                                // This is a placeholder - the actual type name will come from the value itself
                                let indent_next = "  ".repeat(depth + 1);
                                print!("{}  {}->", indent, #field_name_str);
                                
                                // Call the recursive method, which will print the opening brace
                                value.pointer_debug_internal(mem, depth + 1, max_depth, visited_addresses);
                            },
                            Err(e) => {
                                println!("{}  {} → Error reading: {}", indent, #field_name_str, e);
                            }
                        }
                    } else {
                        println!("{}  {} → Already visited address {:#x}", indent, #field_name_str, address);
                    }
                }
            } else {
                quote! {
                    println!("{}  {}: {} = {:?}", indent, #field_name_str, #field_type, self.#field_name);
                }
            }
        })
    } else {
        // Return empty token stream if input is not a struct
        return TokenStream::from(quote! {
            compile_error!("PointerDerefDebugPrint can only be derived for structs");
        });
    };
    
    // Generate the DerefDebugPrint implementation with proper generics and bounds
    let expanded = quote! {
        impl #impl_generics ::memflow_pointer_debug::DerefDebugPrint for #name #ty_generics #new_where_clause {
            fn pointer_debug_internal<M: ::memflow::mem::MemoryView>(
                &self, 
                mem: &mut M, 
                depth: usize, 
                max_depth: usize,
                visited_addresses: &mut ::std::collections::HashSet<u64>
            ) {
                if depth >= max_depth {
                    return;
                }
                
                let indent = "  ".repeat(depth);
                
                // If this is the first level (depth > 0), print without a newline
                if depth > 0 {
                    println!(" {}", stringify!(#name));
                } else {
                    println!("{}{} {{", indent, stringify!(#name));
                }
                
                #(#field_debugs)*
                
                println!("{}}}", indent);
            }
        }
    };
    
    TokenStream::from(expanded)
}