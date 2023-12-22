#[cfg(test)]
use crate::lang::tests::helpers::compare_parsed_to_expected;

#[cfg(test)]
use serde_json::json;

#[cfg(test)]
use crate::assert_parsing;

#[cfg(test)]
assert_parsing! {
    plain: {
        "SELECT * from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "collection": null
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },

    collection: {
        "SELECT users.* from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "collection": "users"
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    mixed_0: {
        "SELECT id, users.name as username from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "expr": {
                                        "type": "Expr::Variable",
                                        "name": "id"
                                    },
                                    "alias": null
                                },
                                {
                                    "expr": {
                                        "type": "Expr::Get",
                                        "object": {
                                            "type": "Expr::Variable",
                                            "name": "users"
                                        },
                                        "name": "name"
                                    },
                                    "alias": "username"
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    mixed_1: {
        "SELECT 5 as five, \"text\" as some_text  from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "expr": {
                                        "type": "Expr::Literal",
                                        "value": "Num(5.0)",
                                        "raw": "5"
                                    },
                                    "alias": "five"
                                },
                                {
                                    "expr": {
                                        "type": "Expr::Literal",
                                        "value": "Str(\"text\")",
                                        "raw": "text"
                                    },
                                    "alias": "some_text"
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    mixed_2: {
        "SELECT 5 + 27 as addition, 4 / 2 as division from users;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(5.0)",
                                            "raw": "5"
                                        },
                                        "operator": {
                                            "Symbol": "Plus"
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(27.0)",
                                            "raw": "27"
                                        }
                                    },
                                    "alias": "addition"
                                },
                                {
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(4.0)",
                                            "raw": "4"
                                        },
                                        "operator": {
                                            "Symbol": "Slash"
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(2.0)",
                                            "raw": "2"
                                        }
                                    },
                                    "alias": "division"
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    },
    mixed_no_from: {
        "SELECT 5 + 27 as addition, 4 / 2 as division;" => {
            "type": "Stmt::Program",
            "body": [
                {
                    "type": "Stmt::Expression",
                    "expr": {
                        "type": "Expr::Select",
                        "value": {
                            "core": {
                                "projection": [{
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(5.0)",
                                            "raw": "5"
                                        },
                                        "operator": {
                                            "Symbol": "Plus"
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(27.0)",
                                            "raw": "27"
                                        }
                                    },
                                    "alias": "addition"
                                },
                                {
                                    "expr": {
                                        "type": "Expr::Binary",
                                        "left": {
                                            "type": "Expr::Literal",
                                            "value": "Num(4.0)",
                                            "raw": "4"
                                        },
                                        "operator": {
                                            "Symbol": "Slash"
                                        },
                                        "right": {
                                            "type": "Expr::Literal",
                                            "value": "Num(2.0)",
                                            "raw": "2"
                                        }
                                    },
                                    "alias": "division"
                                }]
                            },
                            "compound": [],
                            "limit": null,
                            "order_by": null
                        }
                    }
                }
            ]
        }
    }
}