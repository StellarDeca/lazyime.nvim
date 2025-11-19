mod language;

use language::*;
use crate::core::*;
use std::collections::HashMap;
use tree_sitter::{Node, Query, QueryCursor, Range, StreamingIterator, Tree};

pub(super) struct Parser {
    trees: HashMap<String, Option<Tree>>,
    parsers: HashMap<String, tree_sitter::Parser>,
    query: HashMap<String, Query>,
}
impl Parser {
    pub(super) fn new() -> Parser {
        let parsers = HashMap::new();
        let query = HashMap::new();
        let trees = HashMap::new();
        Parser { parsers, query, trees }
    }

    pub(super) fn add_language(&mut self, type_: &SupportLanguage) {
        let adapter = Adapter::adapter(&type_);
        let mut parser = tree_sitter::Parser::new();
        let query = adapter.get_comment_query();

        parser.set_language(&adapter.get_language()).unwrap();
        self.parsers.insert(type_.to_string(), parser);
        self.query.insert(type_.to_string(), query);
    }

    pub(super) fn update_tree(&mut self, type_: &SupportLanguage, code: &String) {
        // 如果tree不存在，则自动新建树
        let tree: Option<Tree>;
        let type_ = type_.to_string();
        let parser = self.parsers.get_mut(&type_).unwrap();
        if let Some(old_tree) = self.trees.get(&type_) {
            tree = parser.parse(code.as_bytes(), Option::from(old_tree));
        } else {
            tree = parser.parse(code.as_bytes(), None);
        }
        self.trees.insert(type_, tree);
    }

    pub(super) fn get_comments(&mut self, type_: &SupportLanguage, code: &String, ) -> Option<NodesRange> {
        let type_ = type_.to_string();
        if let Some(tree) = self.trees.get(&type_).unwrap() {
            let root = tree.root_node();
            let query = self.query.get(&type_).unwrap();
            let mut query_cursor = QueryCursor::new();
            let mut res = query_cursor.matches(&query, root, code.as_bytes());
            // 遍历结果，返回comment的range数组
            let mut node_range = NodesRange::new();
            while let Some(m) = res.next() {
                for iter in m.captures {
                    node_range.add_node(iter.node)
                }
            }
            Some(node_range)
        } else {
            None
        }
    }
}

pub(super) struct NodesRange {
    nodes_range: Vec<Range>,
}
impl NodesRange {
    fn new() -> NodesRange { NodesRange { nodes_range: vec![] } }

    fn add_node(&mut self, node: Node) {
        self.nodes_range.push(node.range())
    }

    pub(super) fn in_range(&self, cursor: &Cursor) -> bool {
        // 判断cursor的位置是否在node节点里。注意 坐标都是 UTF-16字符坐标
        let (sr, sc) = (cursor.row, cursor.column);

        fn cmp_pos(r1: usize, c1: usize, r2: usize, c2: usize) -> i8 {
            // 判断给定的r1, c1是否在r2,c2范围内
            // 范围左面返回-1,范围右面返回1,相等返回0
            if r1 < r2 { return -1 };
            if r1 > r2 { return 1 };
            if c1 < c2 { return -1 };
            if c1 > c2 { return 1 };
            0
        }
        for range in &self.nodes_range {
            let start = range.start_point;
            let end = range.end_point;
            let (rs, cs) = (start.row, start.column);
            let (re, ce) = (end.row, end.column);

            // 严格判断边界条件， 左开右闭
            // 注意TreeSitter本身范围为 左闭右开区间
            if cmp_pos(sr, sc, rs, cs) > 0 && cmp_pos(sr, sc, re, ce) < 0 {
                return true;
            }
        }
        false
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let code = &r#"
// <-- 行注释
pub fn main() { println!("Hello World!"); }
/**
    <--- 块注释
**/
        "#.to_string();
        let mut parser = Parser::new();
        let lang = &SupportLanguage::Rust;
        parser.add_language(lang);
        parser.update_tree(lang, code);
        let res = parser.get_comments(lang, code).unwrap();
        println!("{:?}", res.nodes_range);
        // 判断 TS 解析是否正常, 检查边界条件 与 内部条件 文档注释同样视为单行注释 或 块注释
        // 单行注释
        assert_eq!(res.in_range(&Cursor::new(1, 0)), false);
        assert_eq!(res.in_range(&Cursor::new(1, 1)), true);
        // 代码片段
        assert_eq!(res.in_range(&Cursor::new(2, 5)), false);
        // 块注释
        assert_eq!(res.in_range(&Cursor::new(3, 0)), false);
        assert_eq!(res.in_range(&Cursor::new(3, 1)), true);
        assert_eq!(res.in_range(&Cursor::new(5, 2)), true);
        assert_eq!(res.in_range(&Cursor::new(5, 3)), false);
    }

}
