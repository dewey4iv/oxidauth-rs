use std::cmp;

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Permission<'a> {
    pub realm: &'a str,
    pub resource: &'a str,
    pub action: &'a str,
}

impl Permission<'_> {
    pub fn can(&self, permissions: &mut Vec<Permission>) -> bool {
        self.get_matching(permissions).is_some()
    }

    pub fn get_matching<'a>(&self, permissions: &'a mut Vec<Permission>) -> Option<&Permission<'a>> {
        // we sort here so we can get the most permissive permission
        permissions.sort();
        // we reverse because rust sorts ASC (least -> greatest)
        permissions.reverse();

        for permission in permissions.iter() {
            if permission == self {
                return Some(permission);
            }

            if 
                compare(self.realm, permission.realm) &&
                compare(self.resource, permission.resource) &&
                compare(self.action, permission.action)
            {
                return Some(permission);
            }
        }

        return None;
    }

    pub fn from_string<'a>(input: &'a str) -> Permission<'a> {
        let parts: Vec<&'a str> = input.split(":").collect();

        if parts.len() < 3 {
            panic!("a permission must have all three parts: '{}'", input);
        }

        for field in parts[0..3].iter() {
            if field.len() == 0 {
                panic!("a permission must have all three parts: '{}'", input);
            }
        }

        Permission {
            realm: parts[0],
            resource: parts[1],
            action: parts[2],
        }
    }
}

fn compare(test: &str, challenge: &str) -> bool {
    if test == challenge {
        return true;
    }

    if challenge == "**" {
        return true;
    }

    let test_parts: Vec<&str> = test.split(".").collect();
    let challenge_parts: Vec<&str> = challenge.split(".").collect();

    if challenge_parts.len() == 0 {
        return false;
    }

    for i in (0..test_parts.len()).collect::<Vec<usize>>() {
        if challenge_parts.len() - 1 < i {
            return false;
        }

        let mut matched = false;

        if challenge_parts[i] == "**" {
            return true;
        }

        if challenge_parts[i] == "*" {
            matched = true;
        }

        if challenge_parts[i] == test_parts[i] {
            matched = true;
        }

        if !matched {
            return false;
        }
    }

    return true;
}

impl PartialOrd for Permission<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Permission<'_> {
    fn cmp(&self, other: &Permission) -> cmp::Ordering {
        if self == other {
            return cmp::Ordering::Equal;
        }

        match sort_part(self.realm, other.realm) {
            None => {},
            Some(cmp::Ordering::Equal) => {},
            Some(result) => return result,
        }

        match sort_part(self.resource, other.resource) {
            None => {},
            Some(cmp::Ordering::Equal) => {},
            Some(result) => return result,
        }

        match sort_part(self.action, other.action) {
            None => {},
            Some(cmp::Ordering::Equal) => {},
            Some(result) => return result,
        }

        return cmp::Ordering::Equal;
    }
}

fn sort_part(left: &str, right: &str) -> Option<cmp::Ordering> {
    if left == right {
        return Some(cmp::Ordering::Equal);
    }

    if left == "**" {
        return Some(cmp::Ordering::Greater);
    }

    if right == "**" {
        return Some(cmp::Ordering::Less);
    }

    if left == "*" {
        return Some(cmp::Ordering::Greater);
    }

    if right == "*" {
        return Some(cmp::Ordering::Less);
    }

    return None
}

impl Into<String> for Permission<'_> {
    fn into(self) -> String {
        [
            self.realm.clone(),
            self.resource.clone(),
            self.action.clone(),
        ].join(":")
    }
}

impl<'a> From<&'a str> for Permission<'a> {
    fn from(input: &'a str) -> Self {
        Permission::from_string(input)
    }
}

impl<'a> From<&&'a str> for Permission<'a> {
    fn from(input: &&'a str) -> Self {
        Permission::from_string(input)
    }
}

impl<'a> From<&'a String> for Permission<'a> {
    fn from(input: &'a String) -> Self {
        Permission::from_string(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity() {
        let _permission = Permission {
            realm: "realm",
            resource: "resource",
            action: "action",
        };
    }

    #[test]
    fn test_sorting() {
        struct Test<'a> {
            left: Permission<'a>,
            right: Permission<'a>,
            output: cmp::Ordering,
        }

        let tests = vec![
            Test {
                left: "**:**:**".into(),
                right: "**:**:**".into(),
                output: cmp::Ordering::Equal,
            },
            Test {
                left: "thing1:thing2:thing3".into(),
                right: "**:**:**".into(),
                output: cmp::Ordering::Less,
            },
            Test {
                left: "**:**:**".into(),
                right: "thing1:thing2:thing3".into(),
                output: cmp::Ordering::Greater,
            },
            Test {
                left: "aa:bb:cc".into(),
                right: "cc:bb:aa".into(),
                output: cmp::Ordering::Equal,
            },
            Test {
                left: "aa:bb:**".into(),
                right: "cc:bb:**".into(),
                output: cmp::Ordering::Equal,
            },
            Test {
                left: "aa:bb:**".into(),
                right: "cc:bb:*".into(),
                output: cmp::Ordering::Greater,
            },
        ];

        for Test { left, right, output } in tests {
            assert_eq!(left.cmp(&right), output)
        }
    }

    #[test]
    fn test_sort_part() {
        use std::cmp::Ordering;

        let greater = sort_part("**", "*");
        assert_eq!(greater, Some(Ordering::Greater));

        let result = sort_part("*", "**");
        assert_eq!(result, Some(Ordering::Less));

        let result = sort_part("**", "**");
        assert_eq!(result, Some(Ordering::Equal));

        let result = sort_part("left", "right");
        assert_eq!(result, None);
    }

    #[test]
   fn test_sort_vec() {
        struct Test<'a> {
            input: Vec<&'a str>,
            output: Vec<&'a str>,
        }

        let tests = vec![
            Test {
                input: vec![
                    "**:**:**",
                    "realm:**:**",
                    "realm:**:action",
                    "realm:resource:**",
                    "realm:resource:*",
                    "realm:resource:action",
                ],
                output: vec![
                    "realm:resource:action",
                    "realm:resource:*",
                    "realm:resource:**",
                    "realm:**:action",
                    "realm:**:**",
                    "**:**:**",
                ],
            },
        ];

        let greater = Permission {
            realm: "realm",
            resource: "resouce",
            action: "**",
        };

        let lesser = Permission {
            realm: "realm",
            resource: "resouce",
            action: "*",
        };

        assert!(greater > lesser);
        assert_eq!(true, greater > lesser);

        let mut list = vec![lesser.clone(), greater.clone()];
        list.sort();
        list.reverse();
        assert_eq!(list, vec![greater, lesser]);

        for Test { input, output } in tests {
            let mut input: Vec<Permission> = input.into_iter().map(|string| string.into()).collect();
            input.sort();
            let input: Vec<String> = input.into_iter().map(|permission| permission.into()).collect();

            assert_eq!(input, output);
        }
    }

    #[test]
    fn test_matching_and_can() {
        struct Test<'a> {
            permission: &'a str,
            permissions: Vec<&'a str>,
            output: Option<Permission<'a>>,
        }

        let tests = vec![
            Test {
                permission: "realm:resource:action",
                permissions: vec![],
                output: None,
            },
            Test {
                permission: "realm:resource:action",
                permissions: vec!["realm:resource:other"],
                output: None,
            },
            Test {
                permission: "realm:resource:action",
                permissions: vec!["realm:resource:action"],
                output: Some("realm:resource:action".into()),
            },
            Test {
                permission: "realm:resource:action",
                permissions: vec!["realm:resource:action", "realm:resource:other"],
                output: Some("realm:resource:action".into()),
            },
            Test {
                permission: "realm:resource:action",
                permissions: vec!["realm:resource:action", "realm:resource:other"],
                output: Some("realm:resource:action".into()),
            },
            Test {
                permission: "realm:resource:action",
                permissions: vec!["realm:resource:**", "realm:resource:other"],
                output: Some("realm:resource:**".into()),
            },
            Test {
                permission: "realm:resource.1:action",
                permissions: vec!["realm:resource:other", "realm:resource.*:**"],
                output: Some("realm:resource.*:**".into()),
            },
            Test {
                permission: "realm:resource.1:action",
                permissions: vec!["realm:resource:other", "realm:resource.*:**"],
                output: Some("realm:resource.*:**".into()),
            },
            Test {
                permission: "realm:resource.1:action",
                permissions: vec!["**:**:**", "realm:resource.2:**"],
                output: Some("**:**:**".into()),
            },
            Test {
                permission: "realm:resource.1:action",
                permissions: vec!["realm:resource.2:**", "**:**:**"],
                output: Some("**:**:**".into()),
            },
        ];

        for Test { permission, permissions, output } in tests.into_iter() {
            let permission: Permission = permission.into();

            let mut permissions: Vec<Permission> = permissions
                .into_iter()
                .map(|string| string.into())
                .collect();

            // test can
            let result = permission.can(&mut permissions);
            if output.is_some() {
                assert!(result);
            } else {
                assert!(!result);
            }
            
            // test matching
            let result = permission.get_matching(&mut permissions);

            match (output, result) {
                (Some(output), Some(result)) => assert_eq!(&output, result),
                (None, Some(result)) => assert_eq!(None, Some(result)),
                (Some(output), None) => assert_eq!(Some(output), None),
                (None, None) => assert!(true),
            }
        }
    }

    #[test]
    fn from_str_and_string() {
        struct Test<'a> {
            input: &'a str,
            result: Permission<'a>,
        }

        let tests = vec![
            Test {
                input: "realm:resource:action",
                result: Permission { realm: "realm", resource: "resource", action: "action" },
            },
        ];

        for Test { input, result } in tests.into_iter() {
            let permission: Permission = input.into();

            assert_eq!(result.realm, permission.realm);
            assert_eq!(result.resource, permission.resource);
            assert_eq!(result.action, permission.action);

            let string_intput: String = input.into();
            let permission: Permission = (&string_intput).into();

            assert_eq!(result.realm, permission.realm);
            assert_eq!(result.resource, permission.resource);
            assert_eq!(result.action, permission.action);
        }
    }

    #[test]
    #[should_panic]
    fn from_str_panic() {
        struct Test<'a> {
            input: &'a str,
        }

        let tests = vec![
            Test {
                input: "realm::",
            },
            Test {
                input: "::",
            },
            Test {
                input: "",
            },
        ];

        for Test { input } in tests.into_iter() {
            let _permission: Permission = input.into();
        }
    }

    #[test]
    fn into_string() {
        struct Test<'a> {
            permission: Permission<'a>,
            result: String,
        }

        let tests = vec![
            Test {
                permission: Permission { realm: "realm", resource: "resource", action: "action" },
                result: "realm:resource:action".into(),
            },
            Test {
                permission: Permission { realm: "", resource: "", action: "" },
                result: "::".into(),
            },
        ];

        for Test { permission, result } in tests.into_iter() {
            let output: String = permission.into();

            assert_eq!(result, output);
        }
    }
}
