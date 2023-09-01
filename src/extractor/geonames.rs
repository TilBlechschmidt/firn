use super::Extractor;
use crate::{
    db::{Attribute, Entity, Rule, Value, Variable, VariableSetExt},
    handle::Handle,
    query,
};
use rstar::{primitives::GeomWithData, RTree};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fmt, path::Path};

type GeonameLocation = GeomWithData<[f64; 2], i64>;

pub struct GeoNames {
    tree: RTree<GeonameLocation>,
    geonames: HashMap<i64, Geoname>,
}

impl GeoNames {
    pub fn load(
        geonames: impl AsRef<Path>,
        hierarchy: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut instance = Self::load_geonames(geonames)?;
        instance.load_hierarchy(hierarchy)?;
        Ok(instance)
    }

    fn load_hierarchy(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .flexible(true)
            .from_path(path)?;

        for result in rdr.into_deserialize() {
            let entry: HierarchyEntry = result?;

            if !self.geonames.contains_key(&entry.parent) {
                continue;
            }

            if let Some(child) = self.geonames.get_mut(&entry.child) {
                child.parent = Some(entry.parent);
            }
        }

        Ok(())
    }

    fn load_geonames(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .flexible(true)
            .from_path(path)?;

        let mut entries = Vec::new();
        let mut geonames = HashMap::new();

        for result in rdr.into_deserialize() {
            let geoname: Geoname = result?;

            if ![FeatureClass::P, FeatureClass::A].contains(&geoname.feature_class) {
                continue;
            }

            entries.push(GeonameLocation::new(
                [geoname.latitude, geoname.longitude],
                geoname.geonameid,
            ));

            geonames.insert(geoname.geonameid, geoname);
        }

        let tree = RTree::bulk_load(entries);

        println!("Imported {} geonames", geonames.len());

        Ok(Self { tree, geonames })
    }

    fn handle_coordinate(&self, handle: &mut Handle, entity: Entity) {
        query!(handle where (?lat, ?lng) match [
            { #entity, :"location/latitude", ?lat },
            { #entity, :"location/longitude", ?lng }
        ] => coordinates);

        // Don't do double work
        if handle
            .get(&entity, &"location/geoname".into())
            .next()
            .is_some()
        {
            return;
        }

        // Parse the coords and find the nearest neighbor
        if let Some((Some(lat), Some(lng))) = coordinates.get(0).map(|c| {
            (
                c.get(&lat).map(|v| v.data().parse::<f64>().ok()).flatten(),
                c.get(&lng).map(|v| v.data().parse::<f64>().ok()).flatten(),
            )
        }) {
            // TODO Filter by distance so we don't get super far away matches if there isn't anything close
            if let Some(neighbor) = self.tree.nearest_neighbor(&[lat, lng]) {
                let geoname = Entity::from(format!("geoname:{}", neighbor.data));
                handle.insert(entity, "location/geoname", geoname);
            }
        }
    }

    fn handle_geoname(&self, handle: &mut Handle, entity: Entity, id: i64) {
        if let Some(geoname) = self.geonames.get(&id) {
            // println!("found geoname {id}");
            // println!("{geoname:?}");

            if handle.get(&entity, &"text/label".into()).next().is_none() {
                handle.insert(entity.clone(), "text/label", geoname.name.clone());
            }

            if let Some(parent_id) = &geoname.parent {
                let parent = Entity::from(format!("geoname:{}", parent_id));
                handle.insert(entity, "relation/parent", parent);
            }
        }
    }
}

impl Extractor for GeoNames {
    fn entry_added(
        &mut self,
        handle: &mut Handle,
        entity: &Entity,
        attribute: &Attribute,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if attribute == &Attribute::from("location/latitude")
            || attribute == &Attribute::from("location/longitude")
        {
            self.handle_coordinate(handle, entity.to_owned());
        }

        // This is a bit of a hack because the extractor is not called for ref-only entities
        if let Value::Reference(referenced) = value {
            if let Some(id) = referenced
                .0
                .strip_prefix("geoname:")
                .map(|id| id.parse::<i64>().ok())
                .flatten()
            {
                self.handle_geoname(handle, referenced.to_owned(), id);
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct HierarchyEntry {
    parent: i64,
    child: i64,
    variant: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Geoname {
    geonameid: i64,
    name: String,
    asciiname: Option<String>,
    alternatenames: Option<String>,
    latitude: f64,
    longitude: f64,
    feature_class: FeatureClass,
    feature_code: String,
    country_code: Option<String>,
    cc2: Option<String>,
    admin1_code: Option<String>,
    admin2_code: Option<String>,
    admin3_code: Option<String>,
    admin4_code: Option<String>,
    population: Option<f64>,
    elevation: Option<i32>,
    dem: Option<f64>,
    timezone: Option<String>,
    modification_date: Option<String>,
    #[serde(skip)]
    parent: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum FeatureClass {
    A, // country, state, region,...
    H, // stream, lake, ...
    L, // parks,area, ...
    P, // city, village,...
    R, // road, railroad
    S, // spot, building, farm
    T, // mountain,hill,rock,...
    U, // undersea
    V, // forest,heath,...
    #[serde(other)]
    Other,
}

impl FeatureClass {
    fn to_string(&self) -> String {
        match *self {
            FeatureClass::A => "A".to_string(),
            FeatureClass::H => "H".to_string(),
            FeatureClass::L => "L".to_string(),
            FeatureClass::P => "P".to_string(),
            FeatureClass::R => "R".to_string(),
            FeatureClass::S => "S".to_string(),
            FeatureClass::T => "T".to_string(),
            FeatureClass::U => "U".to_string(),
            FeatureClass::V => "V".to_string(),
            FeatureClass::Other => "X".to_string(),
        }
    }
}
impl fmt::Display for FeatureClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
