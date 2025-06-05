//! Tests for extended visibility modifiers
//! 
//! This module tests the parsing and resolution of various visibility modifiers
//! including internal, protected, package, and restricted visibility.

use crate::ast::*;
use crate::parser::*;
use crate::module_resolver::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_public_visibility() {
        let input = "pub fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.visibility, Visibility::Public);
        assert_eq!(function.name, "test");
    }

    #[test]
    fn test_parse_internal_visibility() {
        let input = "internal fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.visibility, Visibility::Internal);
        assert_eq!(function.name, "test");
    }

    #[test]
    fn test_parse_protected_visibility() {
        let input = "protected fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.visibility, Visibility::Protected);
        assert_eq!(function.name, "test");
    }

    #[test]
    fn test_parse_package_visibility() {
        let input = "package fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.visibility, Visibility::Package);
        assert_eq!(function.name, "test");
    }

    #[test]
    fn test_parse_private_visibility() {
        let input = "fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.visibility, Visibility::Private);
        assert_eq!(function.name, "test");
    }

    #[test]
    fn test_parse_restricted_crate_visibility() {
        let input = "pub(crate) fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        match function.visibility {
            Visibility::Restricted { restriction } => {
                assert_eq!(restriction, VisibilityRestriction::Crate);
            }
            _ => panic!("Expected restricted visibility"),
        }
    }

    #[test]
    fn test_parse_restricted_super_visibility() {
        let input = "pub(super) fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        match function.visibility {
            Visibility::Restricted { restriction } => {
                assert_eq!(restriction, VisibilityRestriction::Super);
            }
            _ => panic!("Expected restricted visibility"),
        }
    }

    #[test]
    fn test_parse_restricted_module_visibility() {
        let input = "pub(self) fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        match function.visibility {
            Visibility::Restricted { restriction } => {
                assert_eq!(restriction, VisibilityRestriction::Module);
            }
            _ => panic!("Expected restricted visibility"),
        }
    }

    #[test]
    fn test_parse_restricted_path_visibility() {
        let input = "pub(in std.io) fun test(): Int { return 1 }";
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        match function.visibility {
            Visibility::Restricted { restriction } => {
                match restriction {
                    VisibilityRestriction::Path(path) => {
                        assert_eq!(path.segments, vec!["std".to_string(), "io".to_string()]);
                    }
                    _ => panic!("Expected path restriction"),
                }
            }
            _ => panic!("Expected restricted visibility"),
        }
    }

    #[test]
    fn test_parse_module_with_visibility() {
        let input = r#"
            internal mod test_module {
                pub fun public_func(): Int { return 1 }
                internal fun internal_func(): Int { return 2 }
                protected fun protected_func(): Int { return 3 }
            }
        "#;
        
        let mut parser = Parser::new(input);
        let module_decl = parser.parse_module_declaration().unwrap();
        
        assert_eq!(module_decl.visibility, Visibility::Internal);
        assert_eq!(module_decl.items.len(), 3);
        
        // Check function visibilities
        for item in &module_decl.items {
            if let Item::Function(func) = item {
                match func.name.as_str() {
                    "public_func" => assert_eq!(func.visibility, Visibility::Public),
                    "internal_func" => assert_eq!(func.visibility, Visibility::Internal),
                    "protected_func" => assert_eq!(func.visibility, Visibility::Protected),
                    _ => panic!("Unexpected function name"),
                }
            }
        }
    }

    #[test]
    fn test_visibility_checker_public() {
        let current_context = ModulePath::single("test".to_string());
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context, package_root);
        
        let item_module = ModulePath::single("other".to_string());
        assert!(checker.is_accessible(&Visibility::Public, &item_module));
    }

    #[test]
    fn test_visibility_checker_private_same_module() {
        let current_context = ModulePath::single("test".to_string());
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context.clone(), package_root);
        
        // Same module - should be accessible
        assert!(checker.is_accessible(&Visibility::Private, &current_context));
        
        // Different module - should not be accessible
        let other_module = ModulePath::single("other".to_string());
        assert!(!checker.is_accessible(&Visibility::Private, &other_module));
    }

    #[test]
    fn test_visibility_checker_internal() {
        let current_context = ModulePath::single("test".to_string());
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context, package_root);
        
        let item_module = ModulePath::single("other".to_string());
        // Internal should be accessible within the same package (for now, always true)
        assert!(checker.is_accessible(&Visibility::Internal, &item_module));
    }

    #[test]
    fn test_visibility_checker_protected() {
        let current_context = ModulePath::new(vec!["parent".to_string(), "child".to_string()]);
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context, package_root);
        
        // Parent module should be accessible (protected allows access from submodules)
        let parent_module = ModulePath::single("parent".to_string());
        assert!(checker.is_accessible(&Visibility::Protected, &parent_module));
        
        // Sibling module should not be accessible
        let sibling_module = ModulePath::single("sibling".to_string());
        assert!(!checker.is_accessible(&Visibility::Protected, &sibling_module));
    }

    #[test]
    fn test_visibility_checker_restricted_crate() {
        let current_context = ModulePath::single("test".to_string());
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context, package_root);
        
        let item_module = ModulePath::single("other".to_string());
        let restriction = VisibilityRestriction::Crate;
        let visibility = Visibility::Restricted { restriction };
        
        // Should be accessible within the same crate
        assert!(checker.is_accessible(&visibility, &item_module));
    }

    #[test]
    fn test_visibility_checker_restricted_super() {
        let current_context = ModulePath::new(vec!["parent".to_string(), "child".to_string()]);
        let package_root = PathBuf::from("/test");
        let checker = VisibilityChecker::new(current_context, package_root);
        
        let parent_module = ModulePath::single("parent".to_string());
        let restriction = VisibilityRestriction::Super;
        let visibility = Visibility::Restricted { restriction };
        
        // Child should be able to access parent's super-restricted items
        assert!(checker.is_accessible(&visibility, &parent_module));
    }

    #[test]
    fn test_visibility_display() {
        assert_eq!(format!("{}", Visibility::Public), "pub");
        assert_eq!(format!("{}", Visibility::Private), "");
        assert_eq!(format!("{}", Visibility::Internal), "internal");
        assert_eq!(format!("{}", Visibility::Protected), "protected");
        assert_eq!(format!("{}", Visibility::Package), "package");
        
        let restricted = Visibility::Restricted { 
            restriction: VisibilityRestriction::Crate 
        };
        assert_eq!(format!("{}", restricted), "pub(crate)");
    }

    #[test]
    fn test_visibility_restriction_display() {
        assert_eq!(format!("{}", VisibilityRestriction::Crate), "crate");
        assert_eq!(format!("{}", VisibilityRestriction::Super), "super");
        assert_eq!(format!("{}", VisibilityRestriction::Module), "self");
        
        let path = VisibilityRestriction::Path(ModulePath::new(vec!["std".to_string(), "io".to_string()]));
        assert_eq!(format!("{}", path), "in std.io");
    }
} 