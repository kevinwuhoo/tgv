use crate::error::TGVError;
use crate::models::contig::Contig;
use crate::models::reference::Reference;
use csv::Reader;
use std::io::BufReader;

// Include the csv files directly as static strings
static HG19_CYTOBAND: &[u8] = include_bytes!("../resources/hg19_cytoband.csv");
static HG38_CYTOBAND: &[u8] = include_bytes!("../resources/hg38_cytoband.csv");

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Stain {
    Gneg,
    Gpos25,
    Gpos50,
    Gpos75,
    Gpos100,
    Acen,
    Gvar,
    Stalk,
    Other,
}

impl Stain {
    fn from(s: &str) -> Result<Self, TGVError> {
        match s {
            "gneg" => Ok(Stain::Gneg),
            "gpos25" => Ok(Stain::Gpos25),
            "gpos50" => Ok(Stain::Gpos50),
            "gpos75" => Ok(Stain::Gpos75),
            "gpos100" => Ok(Stain::Gpos100),
            "acen" => Ok(Stain::Acen),
            "gvar" => Ok(Stain::Gvar),
            "stalk" => Ok(Stain::Stalk),
            _ => Err(TGVError::ValueError(format!("Invalid stain: {}", s))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CytobandSegment {
    pub contig: Contig,
    pub start: usize, // 1-based, inclusive
    pub end: usize,   // 1-based, inclusive
    pub name: String,
    pub stain: Stain,
}

#[derive(Debug, Clone)]
pub struct Cytoband {
    pub reference: Option<Reference>,
    pub contig: Contig,
    pub segments: Vec<CytobandSegment>,
}

impl Cytoband {
    pub fn start(&self) -> usize {
        1
    }

    pub fn end(&self) -> usize {
        self.segments.last().unwrap().end
    }

    pub fn length(&self) -> usize {
        self.end()
    }
}

impl Cytoband {
    pub fn from_reference(reference: &Reference) -> Result<Vec<Self>, TGVError> {
        let mut cytobands: Vec<Cytoband> = Vec::new();

        let content = match reference {
            Reference::Hg19 => HG19_CYTOBAND,
            Reference::Hg38 => HG38_CYTOBAND,
        };

        let reader = BufReader::new(content);
        let mut csv_reader = Reader::from_reader(reader);

        for result in csv_reader.records() {
            let record = result.map_err(|e| TGVError::ParsingError(e.to_string()))?;

            // only keep chr + digits
            let contig_string = record[0].to_string();
            if !(contig_string.starts_with("chr") && contig_string[3..].parse::<usize>().is_ok()) {
                continue;
            }

            let contig = Contig::chrom(&contig_string);
            let start = record[1]
                .parse::<usize>()
                .map_err(|e| TGVError::ParsingError(e.to_string()))?;
            let end = record[2]
                .parse::<usize>()
                .map_err(|e| TGVError::ParsingError(e.to_string()))?;
            let name = record[3].to_string();
            let stain =
                Stain::from(&record[4]).map_err(|e| TGVError::ParsingError(e.to_string()))?;

            let segment = CytobandSegment {
                contig: contig,
                start: start + 1,
                end: end,
                name: name,
                stain: stain,
            };

            if cytobands.is_empty() || cytobands.last().unwrap().contig != segment.contig {
                let cytoband = Cytoband {
                    reference: Some(reference.clone()),
                    contig: segment.contig.clone(),
                    segments: Vec::new(),
                };
                cytobands.push(cytoband);
            }

            cytobands.last_mut().unwrap().segments.push(segment);
        }
        Ok(cytobands)
    }

    pub fn from_non_reference(
        contigs: &Vec<Contig>,
        lengths: Vec<usize>,
    ) -> Result<Vec<Self>, TGVError> {
        Ok(contigs
            .iter()
            .zip(lengths.iter())
            .map(|(contig, length)| Cytoband {
                reference: None,
                contig: contig.clone(),
                segments: vec![CytobandSegment {
                    contig: contig.clone(),
                    start: 1,
                    end: *length,
                    name: "".to_string(),
                    stain: Stain::Other,
                }],
            })
            .collect())
    }
}
