---
title: "Software That Must Not Be Wrong: Property Tests for a Pediatric Dosing Calculator"
date: "2026-08-30"
description: "A pediatric Favipiravir dosing calculator where the code must not be wrong."
tags: [rust, testing, wasm, leptos]
author: "suradet-ps"
---

This article comes from a small project of mine called favi-child (https://github.com/suradet-ps/favi-child), a calculator for pediatric Favipiravir suspension dosing. The brief sounds deceptively simple. Take a child's weight and tell the caregiver how to prepare each dose. But once I started writing the code, I found a layer of responsibility hidden behind every line. If I got the math wrong, the person affected would be a child whose family measures the medicine following my instructions.

The Favipiravir at my hospital comes only as 200 mg tablets, and there is no ready-made liquid formulation for children. The practical approach is to crush the tablet, dissolve it in water, and draw out the exact volume needed. The per-dose amount is weight-based. Day one gets 35 mg/kg, and days two to five get 15 mg/kg. Take a 4.6 kg child. The day-one dose is 161 mg. Using one tablet dissolved in 5 mL of water gives a concentration of 40 mg/mL, which means drawing 4.025 mL. But a syringe does not have markings that fine. It is graduated in half-milliliters. What is actually feasible is drawing 4.0 mL, delivering 160 mg, off by 1 mg.

This is exactly where the project got interesting. The code does not try to find a single formula that answers every case, because what we really need is a plan that can actually be measured, not pretty numbers. The plan must be drawable with a real syringe, fit in the vessel being prepared, use the fewest tablets possible, and when several options exist, pick the one with the smallest deviation. So I wrote it as a search instead. I enumerate pairs of tablet counts and water volumes that pass all the conditions, then pick the best one.

The real code in `src/domain/plan.rs` looks like this

```rust
let mut candidates: Vec<MixingPlan> = Vec::new();
let mut tablets = TABLET_FRACTION_STEP;
while tablets <= max_tablets {
  for &diluent_ml in &DILUENT_VOLUMES_ML {
    if !is_valid_candidate(dose_mg, tablets, diluent_ml) {
      continue;
    }
    // ...compute the draw volume and push into candidates
  }
  tablets += TABLET_FRACTION_STEP;
}
```

Then it ranks them

```rust
candidates.sort_by(|a, b| {
  a.diluent_ml.total_cmp(&b.diluent_ml)
    .then_with(|| a.tablets.total_cmp(&b.tablets))
    .then_with(|| whole_ml(b.draw_ml).cmp(&whole_ml(a.draw_ml)))
    .then_with(|| a.delta_mg.abs().total_cmp(&b.delta_mg.abs()))
});
```

5 mL comes before 10 mL, fewer tablets first, whole numbers first, smallest deviation last. f64 does not implement `Ord` in Rust, so ordering with `total_cmp` is a declaration of how we sort when the numbers are too precise for ordinary comparison.

But the line I am most proud of in this project is not the search or the ranking. It is the line that says that if no plan satisfies the constraints, we return an error out in the open.

```rust
candidates.into_iter().next().ok_or(PlanError::NoSafePlan { dose_mg })
```

It may read like an ordinary line of code, but in the context of a medical tool it is a declaration. This software will never quietly answer "draw 0.6 mL" when that cannot actually be measured. It will always say that no plan exists that satisfies the application's constraints and can be measured in practice. And when that happens, the screen shows a warning instead of a plan card, because our UI is a state machine that maps error states from the start, `Waiting`, `InvalidWeight`, `NoSafePlan`, `Ready`. The requirement that the software must not be wrong is enforced both in the computation layer and in the presentation layer.

Now for the part I most wanted to write about, the tests. One or two examples are not enough for this kind of calculation, because the code accepts weights from newborns up to implausible numbers, and every point must not break. There is no way to write individual test cases for all of them. So I wrote a property-style test that sweeps weights from 0.1 kg to 150 kg in 0.1 kg steps and checks invariants every time.

```rust
for weight in (1..=1500).map(|t| t as f64 / 10.0) {
  match plan_for_weight(weight) {
    Ok(plan) => {
      assert_invariants(&plan.day1, weight, "day1");
      assert_invariants(&plan.days2_5, weight, "days2_5");
    }
    Err(PlanError::NoSafePlan { .. }) => {
      assert!(weight < 0.5, "unexpected NoSafePlan for {weight} kg");
    }
    Err(other) => panic!("unexpected error for {weight} kg: {other}"),
  }
}
```

What are those invariants? The draw volume must be a multiple of 0.5 mL, must not exceed the prepared water volume, and must be at least 1 mL. The tablet count must be a multiple of half a tablet. The delivered dose must equal the drawn volume times the concentration, and the deviation must stay within tolerance. Every medical requirement is translated into an assert line.

```rust
assert!(plan.draw_ml >= MIN_DRAW_VOLUME_ML && plan.draw_ml <= plan.diluent_ml);
assert!((plan.draw_ml * 2.0).fract().abs() < 1e-9);
```

The interesting part is that the test never asks whether a 4.6 kg child should receive 4.0 mL. That is a clinical judgment, not something we decide in a test. It asks whether every generated plan obeys the rules, regardless of the child's weight. That is what makes it a property.

I asked myself why not use proptest, the standard Rust tool for property testing. The answer is that the input space of this problem is small enough to enumerate completely. The weights a user types in are one-decimal numbers, and the plausible range is roughly 0.1 to 150 kg, which is 1,500 values in total, nothing more. Sweeping all of them is feasible and gives a stronger guarantee than sampling. It is not a case of being "likely to pass". It passes every value that actually exists. proptest explores the input space by generating test cases and shrinking failures to minimal counterexamples, and sometimes needs regression files kept with the project. That is extremely valuable when the input space is large or difficult to enumerate, but unnecessary when the entire domain contains only 1,500 valid inputs. For this problem, straightforward enumeration is more complete, more deterministic, gives the same result on every CI run, and is easier to read.

Even more interesting is a test that does not just check the output. It re-runs the whole search to prove that no better candidate was skipped. It checks two things. First, no usable plan exists in a smaller water volume than the one chosen. Second, within the same water volume, no plan uses fewer tablets. Every time I change the ranking criteria or the filtering conditions, this test catches it immediately if the change makes the algorithm pick a worse plan, even when the result still satisfies the conditions. This is testing the optimality of a decision, not just its correctness, a level I never thought I would be able to test.

Part of what makes all this possible is a clear separation in the code. The domain code under `src/domain/` is pure Rust with no knowledge of Leptos or WASM at all. So `cargo test` runs natively on the machine right away, with no wasm build and no browser. The medical logic is tested as ordinary logic, and Leptos is just the shell that renders it.

This separation is not only about a fast test cycle. It lets Rust's type system do the work for us. The domain structure is encoded as types. The `RegimenDay` enum binds the dose constant of each period to itself, and the `RegimenPlan` struct guarantees that a plan always consists of both a day-one plan and a days-two-to-five plan. Combined with `Result<RegimenPlan, PlanError>`, every code path must handle failure explicitly. If I forget something, the compiler refuses to let it through at compile time, not at runtime. That is what makes me comfortable relying on property tests as the last line of defense, because the line before it is the compiler itself.

While writing the constants I ran into another truth. Several numbers in this project are still awaiting pharmacist confirmation, like the rounding tolerance. I wrote them into the code with a note that the value is provisional. That made me feel that property tests are not just a bug-prevention tool. They are a promise the software makes to its users. It says that we know what correctness means for us, and we check it on every run. For software that involves people's lives, having that promise is worth more than code that looks clever.
