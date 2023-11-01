use std::marker::PhantomData;
use halo2_proofs::{ arithmetic::Field, circuit::*, plonk::*, poly::Rotation };

#[derive(Clone, Copy, Debug)]
struct SumConfig {
  pub col_a: Column<Advice>,
  pub col_b: Column<Advice>,
  pub col_c: Column<Advice>,
  pub selector: Selector,
  pub instance: Column<Instance>,
}

#[derive(Debug, Clone)]
struct SumChip<F: Field> {
  config: SumConfig,
  _marker: PhantomData<F>,
}

impl<F: Field> SumChip<F> {
  pub fn construct(config: SumConfig) -> Self {
    Self {
      config,
      _marker: PhantomData,
    }
  }

  pub fn configure(meta: &mut ConstraintSystem<F>) -> SumConfig {
    let col_a = meta.advice_column();
    let col_b = meta.advice_column();
    let col_c = meta.advice_column();
    let selector = meta.selector();
    let instance = meta.instance_column();

    meta.enable_equality(col_a);
    meta.enable_equality(col_b);
    meta.enable_equality(col_c);
    meta.enable_equality(instance);

    meta.create_gate("add", |meta| {
      let s = meta.query_selector(selector);
      let a = meta.query_advice(col_a, Rotation::cur());
      let b = meta.query_advice(col_b, Rotation::cur());
      let c = meta.query_advice(col_c, Rotation::cur());
      vec![s * (a + b - c)]
    });

    SumConfig {
      col_a,
      col_b,
      col_c,
      selector,
      instance,
    }
  }

  pub fn assign_row(
    &self,
    mut layouter: impl Layouter<F>
  ) -> Result<(AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>), Error> {
    layouter.assign_region(
      || "first row",
      |mut region| {
        self.config.selector.enable(&mut region, 0)?;

        let a_cell = region.assign_advice_from_instance(
          || "a",
          self.config.instance,
          0,
          self.config.col_a,
          0
        )?;

        let b_cell = region.assign_advice_from_instance(
          || "f(1)",
          self.config.instance,
          1,
          self.config.col_b,
          0
        )?;

        let c_cell = region.assign_advice(
          || "a + b",
          self.config.col_c,
          0,
          || a_cell.value().copied() + b_cell.value()
        )?;

        Ok((a_cell, b_cell, c_cell))
      }
    )
  }

  pub fn expose_public(
    &self,
    mut layouter: impl Layouter<F>,
    cell: &AssignedCell<F, F>,
    row: usize
  ) -> Result<(), Error> {
    layouter.constrain_instance(cell.cell(), self.config.instance, row)
  }
}

#[derive(Default)]
struct SumCircuit<F>(PhantomData<F>);

impl<F: Field> Circuit<F> for SumCircuit<F> {
  type Config = SumConfig;
  type FloorPlanner = SimpleFloorPlanner;

  fn without_witnesses(&self) -> Self {
    Self::default()
  }

  fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
    SumChip::configure(meta)
  }

  fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
    let chip = SumChip::construct(config);

    let (a, b, c) = chip.assign_row(layouter.namespace(|| "first row"))?;


    chip.expose_public(
      layouter.namespace(|| "c"),
      &c,
      2
    )?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::marker::PhantomData;

  use super::SumCircuit;
  use halo2_proofs::{ dev::MockProver, pasta::Fp };

  #[test]
  fn sum_example() {
    let k = 4;

    let a = Fp::from(10);
    let b = Fp::from(18);
    let out = Fp::from(28);

    let circuit = SumCircuit(PhantomData);

    let mut public_input = vec![a, b, out];

    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();
  }
}