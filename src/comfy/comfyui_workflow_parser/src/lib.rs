use std::fmt;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{self, Visitor, SeqAccess, MapAccess};

#[derive(Serialize, Deserialize)]
pub struct Workflow {
    //pub data: Option<Value>,
    pub nodes: Vec<Node>,
    /// links format:
    /// 0. link ID
    /// 1. source node ID
    /// 2. source node's output slot index
    /// 3. target node ID
    /// 4. target node's input slot index
    /// 5. data type of the link (str)
    pub links: Vec<(i64, i64, i64, i64, i64, String)>
}

#[derive(Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    #[serde(rename = "type")]
    pub node_type: String,
    /// x, y
    pub pos: TupleValues,
    /// width, height
    pub size: TupleValues,
    // pub flags: ??
    pub order: i64,
    pub mode: i64,
    pub inputs: Option<Vec<NodeInput>>,
    pub outputs: Option<Vec<NodeOutput>>,
    // pub properties: ??
    pub widgets_values: Option<Vec<WidgetValue>>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInput {
    pub name: String,
    #[serde(rename = "type")]
    pub input_type: NodeType,
    pub link: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeOutput {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: NodeType,
    pub links: Option<Vec<i64>>,
    pub slot_index: Option<i64>,
}

#[derive(Serialize)]
/// ComfyUI sometimes stores coords / size as an array of two numbers, sometimes as an object
/// with "0" and "1" keys. We have to write our own deserializer for this case.
pub struct TupleValues (f64, f64);

struct NodeSizeVisitor;

impl<'de> Visitor<'de> for NodeSizeVisitor {
    type Value = TupleValues;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a tuple or a map")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<TupleValues, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let size_0 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let size_1 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
        Ok(TupleValues(size_0, size_1))
    }

    fn visit_map<A>(self, mut map: A) -> Result<TupleValues, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut size_0 = None;
        let mut size_1 = None;

        while let Some(key) = map.next_key()? {
            match key {
                "0" => size_0 = Some(map.next_value()?),
                "1" => size_1 = Some(map.next_value()?),
                _ => {}
            }
        }

        let size_0 = size_0.ok_or_else(|| de::Error::missing_field("0"))?;
        let size_1 = size_1.ok_or_else(|| de::Error::missing_field("1"))?;

        Ok(TupleValues(size_0, size_1))
    }
}

impl<'de> Deserialize<'de> for TupleValues {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NodeSizeVisitor)
    }
}

#[derive(Serialize)]
/// Node input and output types are usually strings, but in some edge cases (trigger inputs and
/// outputs) they can be -1.
pub enum NodeType {
    StrType(String),
    IntType(i64),
}

struct NodeTypeVisitor;

impl<'de> Visitor<'de> for NodeTypeVisitor {
    type Value = NodeType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an integer")
    }

    fn visit_i64<E>(self, value: i64) -> Result<NodeType, E>
    where
        E: de::Error,
    {
        Ok(NodeType::IntType(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<NodeType, E>
    where
        E: de::Error,
    {
        Ok(NodeType::StrType(String::from(value)))
    }
}

impl<'de> Deserialize<'de> for NodeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NodeTypeVisitor)
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum WidgetValue {
    StrType(String),
    IntType(i64),
    FloatType(f64),
    BoolType(bool),
}

struct WidgetValueVisitor;

impl<'de> Visitor<'de> for crate::WidgetValueVisitor {
    type Value = WidgetValue;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an integer or a float")
    }

    fn visit_bool<E>(self, value: bool) -> Result<WidgetValue, E>
    where
        E: de::Error,
    {
        Ok(WidgetValue::BoolType(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<WidgetValue, E>
    where
        E: de::Error,
    {
        Ok(WidgetValue::IntType(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<WidgetValue, E>
    where
        E: de::Error,
    {
        Ok(WidgetValue::IntType(value as i64))
    }

    fn visit_f64<E>(self, value: f64) -> Result<WidgetValue, E>
    where
        E: de::Error,
    {
        Ok(WidgetValue::FloatType(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<WidgetValue, E>
    where
        E: de::Error,
    {
        Ok(WidgetValue::StrType(String::from(value)))
    }
}

impl<'de> Deserialize<'de> for WidgetValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(crate::WidgetValueVisitor)
    }
}

#[derive(Debug)]
pub struct Prompts {
    pub positive: Option<Vec<String>>,
    pub negative: Option<Vec<String>>,
}

const OUTPUT_NODE_TYPES: [&str; 1] = [
    "SaveImage",
];

const KSAMPLER_NODE_TYPES: [&str; 1] = [
    "KSampler",
];

const PROMPT_CONTAINING_NODE_TYPES: [&str; 1] = [
    "CLIPTextEncode",
];

impl Workflow {
    pub fn new(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn find_outputs(&self) -> Option<Vec<&Node>> {
        // Iterate through each node and see if it's an output
        let mut outputs: Vec<&Node> = vec![];
        self.nodes.iter().for_each(|node| {
            if OUTPUT_NODE_TYPES.contains(&node.node_type.as_str()){
                    outputs.push(&node)
            }
        });

        if outputs.is_empty() {
            None
        } else {
            Some(outputs)
        }
    }

    pub fn find_prompts_for_node(&self, node: &Node) -> Option<Prompts> {
        // try to find a ksampler node, with positive and negative inputs
        let mut ksampler: Option<&Node> = None;
        let mut order = node.order;

        // if node is already the first order, we can't find any earlier nodes that could have a prompt
        if order == 0 {
            return None;
        }

        while ksampler.is_none() && order != 0 {
            order -= 1;
            // filter up the tree starting from our node ID and finding the next node by "order",
            // until we find a ksampler
            let next_node = self.nodes.iter().find(|n| n.order == order);
            if next_node.is_none() {
                // can't find a next node
                return None;
            }

            if KSAMPLER_NODE_TYPES.contains(&next_node.unwrap().node_type.as_str()) {
                ksampler = next_node;
            }
        }

        // we couldn't find a ksampler
        if ksampler.is_none() {
            return None;
        }

        let ksampler = ksampler.unwrap();

        let mut prompts = Prompts{ positive: None, negative: None };
        // follow the ksampler conditioning inputs back to find the prompts
        for prompt_type in ["positive", "negative"] {
            let input = ksampler.inputs.as_ref().unwrap().iter().find(|i| i.name.as_str() == prompt_type);
            if input.is_some_and(|i| i.link.is_some()) {

                // get the linked node id from the link
                let link = self.links.iter().find(|l| l.0 == input.unwrap().link.unwrap()).unwrap();
                let next_node = self.nodes.iter().find(|n| n.id == link.1).unwrap();

                // make sure the order is lower - prevents infinite loops
                if next_node.order < ksampler.order {
                    if PROMPT_CONTAINING_NODE_TYPES.contains(&next_node.node_type.as_str()) {
                        match prompt_type {
                            "positive" => {
                                if prompts.positive.is_none() {
                                    prompts.positive = Some(vec![match next_node.widgets_values.as_ref().unwrap().first().unwrap() {
                                        WidgetValue::StrType(s) => s.clone(),
                                        _ => panic!("widget was not a string!")
                                    }]);
                                } else {
                                    prompts.positive.as_mut().unwrap().push(match next_node.widgets_values.as_ref().unwrap().first().unwrap() {
                                        WidgetValue::StrType(s) => s.clone(),
                                        _ => panic!("widget was not a string!")
                                    });
                                }
                            }
                            "negative" => {
                                if prompts.negative.is_none() {
                                    prompts.negative = Some(vec![match next_node.widgets_values.as_ref().unwrap().first().unwrap() {
                                        WidgetValue::StrType(s) => s.clone(),
                                        _ => panic!("widget was not a string!")
                                    }]);
                                } else {
                                    prompts.negative.as_mut().unwrap().push(match next_node.widgets_values.as_ref().unwrap().first().unwrap() {
                                        WidgetValue::StrType(s) => s.clone(),
                                        _ => panic!("widget was not a string!")
                                    });
                                }
                            }
                            _ => {}
                        }

                    }
                }


            }
        }

        Some(prompts)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;
    use super::*;

    #[test]
    fn can_create_new() {
        let w = Workflow::new(get_fixture_as_string("simple_workflow.json").as_str()).unwrap();
        assert_eq!(w.nodes.first().unwrap().id, 9);
    }

    #[test]
    fn can_find_output() {
        let w = Workflow::new(get_fixture_as_string("simple_workflow.json").as_str()).unwrap();
        let outputs = w.find_outputs().unwrap();
        assert_eq!(outputs.len(), 1);
        let id = outputs.first().unwrap().id;
        assert_eq!(id, 9);

    }

    #[test]
    fn can_find_prompt() {
        let w = Workflow::new(get_fixture_as_string("simple_workflow.json").as_str()).unwrap();
        let binding = w.find_outputs().unwrap();
        let output = binding.first().unwrap();
        let prompts = w.find_prompts_for_node(output).unwrap();
        assert!(prompts.positive.is_some());
        assert!(prompts.negative.is_some());
        assert_eq!(prompts.positive.clone().unwrap().len(), 1);
        assert_eq!(prompts.negative.clone().unwrap().len(), 1);
        assert_eq!(*prompts.positive.unwrap().first().unwrap(), String::from("beautiful scenery nature glass bottle landscape, , purple galaxy bottle,"));
        assert_eq!(*prompts.negative.unwrap().first().unwrap(), String::from("text, watermark"));
    }

    fn get_fixture_as_string(filename: &str) -> String {
        fs::read_to_string(PathBuf::from_str(format!("tests/fixtures/{}", filename).as_str()).unwrap()).unwrap()
    }
}
