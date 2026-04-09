// Operational Transform implementation
// Simple character-level OT for text editing

use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Operation {
    Insert { position: usize, text: String },
    Delete { position: usize, length: usize },
}

impl Operation {
    pub fn insert(position: usize, text: String) -> Self {
        Self::Insert { position, text }
    }

    pub fn delete(position: usize, length: usize) -> Self {
        Self::Delete { position, length }
    }

    pub fn apply(&self, content: &str) -> String {
        match self {
            Operation::Insert { position, text } => {
                let chars: Vec<char> = content.chars().collect();
                let pos = (*position).min(chars.len());
                let mut result: Vec<char> = Vec::with_capacity(chars.len() + text.chars().count());
                result.extend_from_slice(&chars[..pos]);
                result.extend(text.chars());
                result.extend_from_slice(&chars[pos..]);
                result.into_iter().collect()
            }
            Operation::Delete { position, length } => {
                let chars: Vec<char> = content.chars().collect();
                let pos = (*position).min(chars.len());
                let end = (pos + *length).min(chars.len());
                let mut result: Vec<char> =
                    Vec::with_capacity(chars.len().saturating_sub(end - pos));
                result.extend_from_slice(&chars[..pos]);
                result.extend_from_slice(&chars[end..]);
                result.into_iter().collect()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub op1_prime: Operation,
    pub op2_prime: Operation,
}

pub fn transform(op1: &Operation, op2: &Operation) -> TransformResult {
    match (op1, op2) {
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) => {
            if p1 <= p2 {
                TransformResult {
                    op1_prime: op1.clone(),
                    op2_prime: Operation::Insert {
                        position: p2 + t1.chars().count(),
                        text: t2.clone(),
                    },
                }
            } else {
                TransformResult {
                    op1_prime: Operation::Insert {
                        position: p1 + t2.chars().count(),
                        text: t1.clone(),
                    },
                    op2_prime: op2.clone(),
                }
            }
        }
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Delete {
                position: p2,
                length: l2,
            },
        ) => {
            if p1 <= p2 {
                TransformResult {
                    op1_prime: op1.clone(),
                    op2_prime: Operation::Delete {
                        position: p2 + t1.len(),
                        length: *l2,
                    },
                }
            } else if *p1 >= *p2 + *l2 {
                TransformResult {
                    op1_prime: Operation::Insert {
                        position: p1 - l2,
                        text: t1.clone(),
                    },
                    op2_prime: op2.clone(),
                }
            } else {
                TransformResult {
                    op1_prime: Operation::Insert {
                        position: *p2,
                        text: t1.clone(),
                    },
                    op2_prime: Operation::Delete {
                        position: p2 + t1.len(),
                        length: *l2,
                    },
                }
            }
        }
        (
            Operation::Delete {
                position: p1,
                length: l1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) => {
            if p2 <= p1 {
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: p1 + t2.chars().count(),
                        length: *l1,
                    },
                    op2_prime: op2.clone(),
                }
            } else if *p2 >= *p1 + *l1 {
                TransformResult {
                    op1_prime: op1.clone(),
                    op2_prime: Operation::Insert {
                        position: p2 - l1,
                        text: t2.clone(),
                    },
                }
            } else {
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: *p2,
                        length: *l1 - (p2 - p1),
                    },
                    op2_prime: Operation::Insert {
                        position: *p1,
                        text: t2.clone(),
                    },
                }
            }
        }
        (
            Operation::Delete {
                position: p1,
                length: l1,
            },
            Operation::Delete {
                position: p2,
                length: l2,
            },
        ) => {
            let start1 = *p1;
            let end1 = p1 + l1;
            let start2 = *p2;
            let end2 = p2 + l2;

            if end1 <= start2 {
                TransformResult {
                    op1_prime: op1.clone(),
                    op2_prime: Operation::Delete {
                        position: p2 - l1,
                        length: *l2,
                    },
                }
            } else if end2 <= start1 {
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: p1 - l2,
                        length: *l1,
                    },
                    op2_prime: op2.clone(),
                }
            } else if start1 <= start2 && end1 >= end2 {
                let overlap = min(end1, end2) - max(start1, start2);
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: *p1,
                        length: l1 - overlap,
                    },
                    op2_prime: Operation::Delete {
                        position: *p1,
                        length: 0,
                    },
                }
            } else if start2 <= start1 && end2 >= end1 {
                let overlap = min(end1, end2) - max(start1, start2);
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: start2,
                        length: 0,
                    },
                    op2_prime: Operation::Delete {
                        position: *p2,
                        length: l2 - overlap,
                    },
                }
            } else if start1 < start2 {
                let overlap = min(end1, end2) - max(start1, start2);
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: *p1,
                        length: (start2 - start1),
                    },
                    op2_prime: Operation::Delete {
                        position: start2 - (start2 - start1),
                        length: l2 - overlap,
                    },
                }
            } else {
                let overlap = min(end1, end2) - max(start1, start2);
                TransformResult {
                    op1_prime: Operation::Delete {
                        position: start2 + (l2 - overlap),
                        length: l1 - overlap,
                    },
                    op2_prime: Operation::Delete {
                        position: *p2,
                        length: (start1 - start2),
                    },
                }
            }
        }
    }
}

pub fn compose(op1: &Operation, op2: &Operation) -> Vec<Operation> {
    match (op1, op2) {
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) if p1 == p2 => {
            vec![Operation::insert(*p1, format!("{}{}", t1, t2))]
        }
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) => {
            let shift = t1.chars().count();
            let new_p2 = if *p2 <= *p1 { *p2 } else { *p2 + shift };
            vec![op1.clone(), Operation::insert(new_p2, t2.clone())]
        }
        (
            Operation::Insert {
                position: p1,
                text: t1,
            },
            Operation::Delete {
                position: p2,
                length: l2,
            },
        ) => {
            let shift = t1.chars().count();
            let new_p2 = if *p2 <= *p1 { *p2 } else { *p2 + shift };
            vec![op1.clone(), Operation::delete(new_p2, *l2)]
        }
        (
            Operation::Delete {
                position: p1,
                length: l1,
            },
            Operation::Insert {
                position: p2,
                text: t2,
            },
        ) => {
            let shift = *l1;
            let new_p2 = if *p2 <= *p1 { *p2 } else { *p2 - shift };
            vec![op1.clone(), Operation::insert(new_p2, t2.clone())]
        }
        (
            Operation::Delete {
                position: p1,
                length: l1,
            },
            Operation::Delete {
                position: p2,
                length: l2,
            },
        ) if p1 == p2 => {
            vec![Operation::delete(*p1, l1 + l2)]
        }
        (
            Operation::Delete {
                position: p1,
                length: l1,
            },
            Operation::Delete {
                position: p2,
                length: l2,
            },
        ) => {
            let end1 = *p1 + l1;
            let end2 = *p2 + l2;
            if end1 >= *p2 && end1 <= end2 {
                vec![Operation::delete(*p1, end2 - p1)]
            } else if end2 >= *p1 && end2 <= end1 {
                let adj_p2 = if *p2 >= end1 { *p2 - l1 } else { *p2 };
                vec![op1.clone(), Operation::delete(adj_p2, *l2)]
            } else {
                let shift = *l1;
                let new_p2 = if *p2 >= end1 { *p2 - shift } else { *p2 };
                vec![op1.clone(), Operation::delete(new_p2, *l2)]
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentState {
    content: String,
    version: u64,
    pending_ops: Vec<(u64, Operation)>,
}

impl DocumentState {
    pub fn new(content: String) -> Self {
        Self {
            content,
            version: 0,
            pending_ops: Vec::new(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn apply(&mut self, op: Operation, client_version: u64) -> Result<u64, String> {
        let mut transformed_op = op;

        for (ver, pending_op) in &self.pending_ops {
            if *ver > client_version {
                let result = transform(&transformed_op, pending_op);
                transformed_op = result.op1_prime;
            }
        }

        self.content = transformed_op.apply(&self.content);
        self.version += 1;
        self.pending_ops
            .push((self.version, transformed_op.clone()));

        while self.pending_ops.len() > 100 {
            self.pending_ops.remove(0);
        }

        Ok(self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_operation() {
        let op = Operation::insert(5, "world ".to_string());
        let result = op.apply("hello!");
        assert_eq!(result, "helloworld !");
    }

    #[test]
    fn test_delete_operation() {
        let op = Operation::delete(5, 6);
        let result = op.apply("hello world!");
        assert_eq!(result, "hello!");
    }

    #[test]
    fn test_insert_utf8() {
        let op = Operation::insert(2, "世界".to_string());
        let result = op.apply("hello");
        assert_eq!(result, "he世界llo");
    }

    #[test]
    fn test_delete_utf8() {
        let op = Operation::delete(2, 2);
        let result = op.apply("hëllo wörld");
        let chars: Vec<char> = "hëllo wörld".chars().collect();
        let expected: String = [&chars[..2], &chars[4..]].concat().into_iter().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_insert_at_boundary() {
        let op = Operation::insert(0, "prefix ".to_string());
        let result = op.apply("hello");
        assert_eq!(result, "prefix hello");
    }

    #[test]
    fn test_insert_past_end() {
        let op = Operation::insert(100, " suffix".to_string());
        let result = op.apply("hello");
        assert_eq!(result, "hello suffix");
    }

    #[test]
    fn test_transform_insert_insert() {
        let op1 = Operation::insert(5, "A".to_string());
        let op2 = Operation::insert(10, "B".to_string());
        let result = transform(&op1, &op2);

        assert_eq!(result.op1_prime, Operation::insert(5, "A".to_string()));
        assert_eq!(result.op2_prime, Operation::insert(11, "B".to_string()));
    }

    #[test]
    fn test_transform_insert_insert_utf8() {
        let op1 = Operation::insert(2, "日本".to_string());
        let op2 = Operation::insert(5, "B".to_string());
        let result = transform(&op1, &op2);
        assert_eq!(result.op2_prime, Operation::insert(7, "B".to_string()));
    }

    #[test]
    fn test_document_state() {
        let mut state = DocumentState::new("hello".to_string());
        let op = Operation::insert(5, " world".to_string());
        let version = state.apply(op, 0).unwrap();
        assert_eq!(state.content(), "hello world");
        assert_eq!(version, 1);
    }

    #[test]
    fn test_compose_insert_insert_same_pos() {
        let op1 = Operation::insert(3, "foo".to_string());
        let op2 = Operation::insert(3, "bar".to_string());
        let result = compose(&op1, &op2);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Operation::insert(3, "foobar".to_string()));
    }

    #[test]
    fn test_compose_insert_insert_different_pos() {
        let op1 = Operation::insert(2, "ab".to_string());
        let op2 = Operation::insert(5, "cd".to_string());
        let result = compose(&op1, &op2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Operation::insert(2, "ab".to_string()));
        assert_eq!(result[1], Operation::insert(7, "cd".to_string()));
    }

    #[test]
    fn test_compose_insert_delete() {
        let op1 = Operation::insert(3, "xyz".to_string());
        let op2 = Operation::delete(5, 2);
        let result = compose(&op1, &op2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Operation::insert(3, "xyz".to_string()));
        assert_eq!(result[1], Operation::delete(8, 2));
    }

    #[test]
    fn test_compose_delete_insert() {
        let op1 = Operation::delete(2, 3);
        let op2 = Operation::insert(5, "new".to_string());
        let result = compose(&op1, &op2);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Operation::delete(2, 3));
        assert_eq!(result[1], Operation::insert(2, "new".to_string()));
    }

    #[test]
    fn test_compose_delete_delete_same_pos() {
        let op1 = Operation::delete(3, 2);
        let op2 = Operation::delete(3, 1);
        let result = compose(&op1, &op2);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Operation::delete(3, 3));
    }
}
