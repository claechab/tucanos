use log::{info, warn};

use crate::{mesh::SimplexMesh, topo_elems::Elem, Error, Idx, Mesh, Result, Tag};

impl<const D: usize, E: Elem> SimplexMesh<D, E> {
    #[cfg(not(feature = "scotch"))]
    pub fn partition_scotch(&mut self, _n_parts: Idx) -> Result<()> {
        Err(Error::from("the scotch feature is not enabled"))
    }

    #[cfg(feature = "scotch")]
    pub fn partition_scotch(&mut self, n_parts: Idx) -> Result<()> {
        if self.etags().any(|t| t != 1) {
            warn!("Erase the element tags");
        }

        info!("Partition the mesh into {} using scotch", n_parts);
        if self.elem_to_elems.is_none() {
            self.compute_elem_to_elems();
        }

        let mut partition = vec![0; self.n_elems() as usize];
        let e2e = self.elem_to_elems.as_ref().unwrap();

        let architecture = scotch::Architecture::complete(n_parts as i32);

        let xadj: Vec<scotch::Num> = e2e
            .ptr
            .iter()
            .copied()
            .map(|x| x.try_into().unwrap())
            .collect();
        let adjncy: Vec<scotch::Num> = e2e
            .indices
            .iter()
            .copied()
            .map(|x| x.try_into().unwrap())
            .collect();

        let mut graph = scotch::Graph::build(&scotch::graph::Data::new(
            0,
            &xadj,
            &[],
            &[],
            &[],
            &adjncy,
            &[],
        ))
        .unwrap();
        graph.check().unwrap();
        graph
            .mapping(&architecture, &mut partition)
            .compute(&mut scotch::Strategy::new())?;

        self.etags = partition.iter().copied().map(|i| i as Tag + 1).collect();

        Ok(())
    }

    #[cfg(not(feature = "metis"))]
    pub fn partition_metis(&mut self, _n_parts: Idx) -> Result<()> {
        Err(Error::from("the metis feature is not enabled"))
    }

    #[cfg(feature = "metis")]
    pub fn partition_metis(&mut self, n_parts: Idx) -> Result<()> {
        if self.etags().any(|t| t != 1) {
            warn!("Erase the element tags");
        }

        info!("Partition the mesh into {} using metis", n_parts);
        if self.elem_to_elems.is_none() {
            self.compute_elem_to_elems();
        }

        let mut partition = vec![0; self.n_elems() as usize];
        let e2e = self.elem_to_elems.as_ref().unwrap();

        let mut xadj: Vec<metis::Idx> = e2e
            .ptr
            .iter()
            .copied()
            .map(|x| x.try_into().unwrap())
            .collect();
        let mut adjncy: Vec<metis::Idx> = e2e
            .indices
            .iter()
            .copied()
            .map(|x| x.try_into().unwrap())
            .collect();

        metis::Graph::new(1, n_parts as metis::Idx, &mut xadj, &mut adjncy)
            .part_recursive(&mut partition)
            .unwrap();

        self.etags = partition.iter().copied().map(|i| i as Tag + 1).collect();

        Ok(())
    }

    pub fn partition_quality(&self) -> Result<f64> {
        if self.faces_to_elems.is_none() {
            return Err(Error::from("face to element connectivity not computed"));
        }

        let f2e = self.faces_to_elems.as_ref().unwrap();

        let n = f2e
            .iter()
            .filter(|(_, v)| v.len() == 2 && self.etags[v[0] as usize] != self.etags[v[1] as usize])
            .count();
        Ok(n as f64 / f2e.len() as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        test_meshes::{test_mesh_2d, test_mesh_3d},
        Result,
    };

    #[cfg(feature = "scotch")]
    #[test]
    fn test_partition_scotch_2d() -> Result<()> {
        let mut mesh = test_mesh_2d().split().split().split().split().split();

        mesh.partition_scotch(4)?;

        let q = mesh.partition_quality()?;
        assert!(q < 0.03);

        Ok(())
    }

    #[cfg(feature = "scotch")]
    #[test]
    fn test_partition_scotch_3d() -> Result<()> {
        let mut mesh = test_mesh_3d().split().split().split().split();

        mesh.partition_scotch(4)?;

        let q = mesh.partition_quality()?;
        assert!(q < 0.025);

        Ok(())
    }

    #[cfg(feature = "metis")]
    #[test]
    fn test_partition_metis_2d() -> Result<()> {
        let mut mesh = test_mesh_2d().split().split().split().split().split();

        mesh.partition_metis(4)?;

        let q = mesh.partition_quality()?;
        assert!(q < 0.03);

        Ok(())
    }

    #[cfg(feature = "metis")]
    #[test]
    fn test_partition_metis_3d() -> Result<()> {
        let mut mesh = test_mesh_3d().split().split().split().split();

        mesh.partition_metis(4)?;

        let q = mesh.partition_quality()?;
        assert!(q < 0.02);

        Ok(())
    }
}
