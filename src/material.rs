use crate::{gameinfo::Gameinfo, keyvalue::KeyValues};

#[derive(Eq, Hash, PartialEq)]
pub enum Material {
    LightmappedGeneric {
        basetexture: String,
    },
    UnlitGeneric {
        basetexture: String,
    },
    WorldVertexTransition {
        basetexture: String,
        basetexture2: String,
    },
    MissingMaterial,
}

impl Material {
    pub fn parse(gameinfo: &Gameinfo, name: &String) -> Option<Self> {
        let kv = KeyValues::parse_from_searchpath(gameinfo, &format!("materials/{}", name), "vmt")
            .ok()?;
        let (shader, properties) = kv.get_all_kv_pairs()[0]; // A material file shouldn't be empty
        match shader.to_lowercase().as_str() {
            "lightmappedgeneric" => {
                let basetexture = properties.get("$basetexture")?.get_value()?.to_lowercase();
                Some(Self::LightmappedGeneric { basetexture })
            }
            "unlitgeneric" => {
                let basetexture = properties.get("$basetexture")?.get_value()?.to_lowercase();
                Some(Self::UnlitGeneric { basetexture })
            }
            "worldvertextransition" => {
                let basetexture = properties.get("$basetexture")?.get_value()?.to_lowercase();
                let basetexture2 = properties.get("$basetexture2")?.get_value()?.to_lowercase();
                Some(Self::WorldVertexTransition {
                    basetexture,
                    basetexture2,
                })
            }
            x => {
                eprintln!("Unknown shader {} in {}", x, name);
                None
            }
        }
    }

    pub fn get_all_textures(&self) -> Vec<&String> {
        match self {
            Material::LightmappedGeneric { basetexture } => vec![basetexture],
            Material::UnlitGeneric { basetexture } => vec![basetexture],
            Material::WorldVertexTransition {
                basetexture,
                basetexture2,
            } => vec![basetexture, basetexture2],
            Material::MissingMaterial => vec![],
        }
    }

    pub fn is_tool(&self) -> bool {
        match self {
            Material::LightmappedGeneric { basetexture } => {
                basetexture.to_lowercase().starts_with("tools/")
            }
            Material::UnlitGeneric { basetexture } => {
                basetexture.to_lowercase().starts_with("tools/")
            }
            Material::WorldVertexTransition {
                basetexture,
                basetexture2,
            } => {
                basetexture.to_lowercase().starts_with("tools/")
                    && basetexture2.to_lowercase().starts_with("tools/")
            }
            Material::MissingMaterial => false,
        }
    }
}
