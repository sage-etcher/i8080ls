
use crate::data_type::FxDashMap;

enum Symbol { 
    // {{{
    NUMBER,
    PAREN_OPEN,     // r0
    PAREN_CLOSE,
    MULT,           // r1
    DIV,
    MOD,
    SHL,
    SHR,
    SUB,            // r2
    ADD,
    NOT,            // r3
    AND,            // r4
    OR,             // r5
    XOR,
    // }}}
}

// {{{
#[derive(Copy)]
struct SymbolItem {
    symbol: Symbol,
    value:  Option<u16>
}

impl SymbolItem {
    fn new(symbol: Symbol, value: Option<u16>) -> Self {
        Self { symbol, value }
    }
}

#[derive(PartialEq)]
enum NodeType {
    // {{{
    NUMBER, // a            // 0
    MULT,   // a * b        // 5
    DIV,    // a / b        // 5
    MOD,    // a MOD b      // 5
    SHL,    // a SHL b      // 5
    SHR,    // a SHR b      // 5
    SUB,    // a - b        // 4
    ADD,    // a + b        // 4
    NOT,    // NOT a        // 3
    AND,    // a AND b      // 2
    OR,     // a OR b       // 1
    XOR,    // a XOR b      // 1
    // }}}
}

impl From::<Symbol> for NodeType {
    fn from(value: Symbol) -> Self {
        // {{{
        match value { 
			Symbol::NUMBER => NodeType::NUMBER,
			Symbol::MULT   => NodeType::MULT,
			Symbol::DIV    => NodeType::DIV,
			Symbol::MOD    => NodeType::MOD,
			Symbol::SHL    => NodeType::SHL,
			Symbol::SHR    => NodeType::SHR,
			Symbol::SUB    => NodeType::SUB,
			Symbol::ADD    => NodeType::ADD,
			Symbol::NOT    => NodeType::NOT,
			Symbol::AND    => NodeType::AND,
			Symbol::OR     => NodeType::OR,
			Symbol::XOR    => NodeType::XOR,
        }
        // }}}
    }
}

struct Node {
    action: NodeType,
    result: Option<u16>,
    left:   Box<Option<Node>>,
    right:  Box<Option<Node>>,
}

// }}}
fn get_operator_precedence() -> FxDashMap<Symbol, u32> {
    // {{{
    let precedence: FxDashMap<Symbol, u32> = FxDashMap::default();
    precedence.insert(Symbol::MULT,   6);
    precedence.insert(Symbol::DIV,    7);
    precedence.insert(Symbol::MOD,    6);
    precedence.insert(Symbol::SHL,    6);
    precedence.insert(Symbol::SHR,    6);
    precedence.insert(Symbol::SUB,    5);
    precedence.insert(Symbol::ADD,    5);
    precedence.insert(Symbol::NOT,    4);
    precedence.insert(Symbol::AND,    3);
    precedence.insert(Symbol::OR,     2);
    precedence.insert(Symbol::XOR,    2);
    precedence.insert(Symbol::NUMBER, 1);

    return precedence;
    // }}}
}

fn build_eval_tree(sym_list: Vec<SymbolItem>) -> Option<Node> {
    // {{{
    if sym_list.len() == 0 {
        return None;
    }

    let precedence_map = get_operator_precedence();

    fn find_top(
        map: FxDashMap<Symbol, u32>,
        sym_list: Vec<SymbolItem>, 
        n: usize, 
        i: usize, 
        mid_i: usize, 
        mid_precedence: u32,
    ) -> (usize, u32) {
        if i == n {
            return (mid_i, mid_precedence);
        }

        let sym = sym_list[i].symbol;
        let precedence = map.get(&sym).unwrap();
        if precedence < mid_precedence {
            return find_top(map, sym_list, n, i+1, i, precedence);
        } else {
            return find_top(map, sym_list, n, i+1, mid_i, mid_precedence);
        }
    }

    let max = sym_list.len();
    let (mid_i, _) = find_top(precedence_map, sym_list, max, 0, 0, 0);
    let mid = &sym_list[mid_i];
    let action = NodeType::from(mid.symbol);
    let left  = Box::new(build_eval_tree(sym_list[0..mid_i].to_vec()));
    let right = Box::new(build_eval_tree(sym_list[mid_i+1..max].to_vec()));
    let result = mid.value;

    return Some(Node { action, result, left, right });
    // }}}
}

fn eval_tree(tree: Option<Node>) -> Option<u16> {
    // {{{
    if tree.is_none() {
        return None;
    }

    if tree.unwrap().action == NodeType::NUMBER {
        return tree.unwrap().result;
    }

    let left_value  = Box::new(eval_tree(tree.unwrap().left.downcast()));
    let right_value = Box::new(eval_tree(tree.unwrap().right.downcast()));

    if right_value.is_none() {
        // all operation require a right value throw error, if not exist
        return None;
    }

    if tree.unwrap().action != NodeType::NOT &&
        left_value.is_none()
    {
        // all non NOT operations require a left value, throw error if missing
        return None;
    }

    match tree.unwrap().action {
        NodeType::MULT => Some(left_value.unwrap() * right_value.unwrap()),
        NodeType::DIV  => Some(left_value.unwrap() / right_value.unwrap()),
        NodeType::MOD  => Some(left_value.unwrap() % right_value.unwrap()),
        NodeType::SUB  => Some(left_value.unwrap() - right_value.unwrap()),
        NodeType::ADD  => Some(left_value.unwrap() + right_value.unwrap()),
        NodeType::AND  => Some(left_value.unwrap() & right_value.unwrap()),
        NodeType::OR   => Some(left_value.unwrap() | right_value.unwrap()),
        NodeType::XOR  => Some(left_value.unwrap() ^ right_value.unwrap()),
        NodeType::SHL  => Some(left_value.unwrap() << right_value.unwrap()),
        NodeType::SHR  => Some(left_value.unwrap() >> right_value.unwrap()),
        NodeType::NOT  => Some(!right_value.unwrap()),
    }
    // }}}
}

fn main() {
    // y = 2 * 3 + 4 * 5
    // y = 6     + 20
    // y = 26
    print!("2 * 3 + 4 * 5 = ");
    let equation = vec![
        SymbolItem::new(Symbol::NUMBER, Some(2)),
        SymbolItem::new(Symbol::MULT,   None),
        SymbolItem::new(Symbol::NUMBER, Some(3)),
        SymbolItem::new(Symbol::ADD,    None),
        SymbolItem::new(Symbol::NUMBER, Some(4)),
        SymbolItem::new(Symbol::MULT,   None),
        SymbolItem::new(Symbol::NUMBER, Some(5)),
    ];

    let syntax_tree = build_eval_tree(equation);
    let result = eval_tree(syntax_tree);
    if result.is_none() {
        println!("\nerror: cannot evaluate equation");
    } else {
        println!("{}", result.unwrap());
    }

}


