use algonaut_core::{Address, MicroAlgos};
use url::Url;
use urlencoding::encode;

pub struct LinkableTransactionBuilder {
    receiver: Address,
    label: Option<String>,
    type_: LinkableTransactionType,
    note: Option<Note>,
}

impl LinkableTransactionBuilder {
    pub fn payment(receiver: Address, amount: MicroAlgos) -> LinkableTransactionBuilder {
        LinkableTransactionBuilder {
            receiver,
            type_: LinkableTransactionType::Payment { amount },
            label: None,
            note: None,
        }
    }

    pub fn asset_transfer(
        receiver: Address,
        asset: u64,
        amount: u64,
    ) -> LinkableTransactionBuilder {
        LinkableTransactionBuilder {
            receiver,
            type_: LinkableTransactionType::AssetTransfer { asset, amount },
            label: None,
            note: None,
        }
    }

    /// Address label (e.g. name of receiver)
    pub fn label<T: Into<String>>(mut self, label: T) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn note(mut self, note: Note) -> Self {
        self.note = Some(note);
        self
    }

    pub fn build(self) -> LinkableTransaction {
        LinkableTransaction {
            receiver: self.receiver,
            label: self.label,
            type_: self.type_,
            note: self.note,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkableTransaction {
    receiver: Address,
    type_: LinkableTransactionType,
    label: Option<String>,
    note: Option<Note>,
}

impl LinkableTransaction {
    pub fn as_url(&self) -> Url {
        let encoded_params = self
            .params()
            .into_iter()
            .map(|(k, v)| (k, encode(&v).into_owned()))
            .collect::<Vec<(String, String)>>();

        Url::parse_with_params(
            &format!("algorand://{}?", self.receiver.to_string()),
            encoded_params,
        )
        // unwrap: we're responsible for ensuring that the URL is valid
        .unwrap()
    }

    fn params(&self) -> Vec<(String, String)> {
        let mut vec = vec![];
        if let Some(label) = &self.label {
            vec.push(("label".to_owned(), label.clone()));
        }
        if let Some(note) = &self.note {
            vec.push(match note {
                Note::Editable(note) => ("note".to_owned(), note.clone()),
                Note::NotEditable(note) => ("xnote".to_owned(), note.clone()),
            })
        }
        vec.extend(match &self.type_ {
            LinkableTransactionType::Payment { amount } => {
                vec![("amount".to_owned(), amount.to_string())]
            }
            LinkableTransactionType::AssetTransfer { asset, amount } => vec![
                ("asset".to_owned(), asset.to_string()),
                ("amount".to_owned(), amount.to_string()),
            ],
        });
        vec
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LinkableTransactionType {
    Payment { amount: MicroAlgos },
    AssetTransfer { asset: u64, amount: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// The note will be editable by the user before submitting the transaction.
    Editable(String),
    /// The note will not be editable by the user.
    NotEditable(String),
}
