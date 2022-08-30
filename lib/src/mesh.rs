use std::collections::BTreeMap;

use geo::MultiLineString;
use topojson::{ArcIndexes, Geometry, Topology, Value};

use crate::{stitch::stitch, translate};

fn mesh_arcs(topology: &Topology) -> topojson::Geometry {
    let n = topology.arcs.len();

    let mut arcs = Vec::with_capacity(n);
    for (i, a) in arcs.iter_mut().enumerate() {
        *a = i as i32;
    }

    topojson::Geometry::new(Value::MultiLineString(stitch(&topology, arcs)))
}

fn mesh_arcs_with_oebject_and_filter(
    topology: &Topology,
    object: &topojson::Geometry,
    filter: (),
) -> topojson::Geometry {
    let arcs = ExtractArcs::default().generate(topology, object, filter);
    // topojson::Geometry::new(Value::MultiLineString(stitch(&topology, arcs)))
    todo!();
}

struct ExtractArcs {
    arcs: Vec<ArcIndexes>,
    geom: Option<Geometry>,
    geosmsByArc: BTreeMap<usize, ArcIndexes>,
    filter: Option<Box<dyn Fn() -> ()>>,
}

impl Default for ExtractArcs {
    fn default() -> Self {
        Self {
            arcs: vec![],
            geom: None,
            geosmsByArc: BTreeMap::new(),
            filter: None,
        }
    }
}
impl ExtractArcs {
    fn extract0(&self, i: i32) {
        let j = translate(i);
    }

    fn extract1(&self, arcs: &ArcIndexes) {
        arcs.iter().for_each(|arc| self.extract0(*arc))
    }

    fn extract2(&self, arcs: &Vec<ArcIndexes>) {
        arcs.iter().for_each(|arc| self.extract1(arc))
    }

    fn extract3(&self, arcs: &Vec<Vec<ArcIndexes>>) {
        arcs.iter().for_each(|arc| self.extract2(arc))
    }

    fn geometry(&mut self, o: &topojson::Geometry) {
        self.geom = Some(o.clone());
        match &o.value {
            Value::GeometryCollection(gc) => {
                for g in gc {
                    self.geometry(g);
                }
            }
            Value::LineString(arcs) => self.extract1(arcs),
            Value::MultiLineString(arcs) => self.extract2(arcs),
            Value::MultiPolygon(arcs) => self.extract3(arcs),
            _ => {}
        }
    }

    fn generate(
        mut self,
        topology: &Topology,
        object: &topojson::Geometry,
        filter: (),
    ) -> Vec<ArcIndexes> {
        self.geometry(object);

        self.geosmsByArc
            .iter()
            .for_each(|geoms| match &self.filter {
                Some(fun) => {
                    todo!();
                }
                None => {
                    todo!();
                }
            });

        self.arcs
    }
}
