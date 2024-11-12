/** Query conditions */

import {
  ConditionType,
  type ConditionSerialized,
  type ConstraintSerialized,
} from "./types";

// ************************************************************************* //
//                             CONDITION CLASSES                             //
// ************************************************************************* //

/** Condition base class. Implements shared methods */
export class Condition {
  /** Serialize the condition to JSON */
  toJSON(): ConditionSerialized {
    throw new Error("Cannot serialize base Condition class");
  }

  /** Create a condition instance from a constraint */
  static fromConstraint(constraint: ConstraintSerialized): Condition {
    return new ConditionSingle(constraint);
  }

  // Helper methods to build ConditionAnd & ConditionOr from an existing condition
  /** Build a new AND condition from an existing condition and an additional constraint. */
  and(constraint: ConstraintSerialized): ConditionAnd {
    return new ConditionAnd([this, Condition.fromConstraint(constraint)]);
  }

  /** Build a new OR condition from an existing condition and an additional constraint. */
  or(constraint: ConstraintSerialized): ConditionOr {
    return new ConditionOr([this, Condition.fromConstraint(constraint)]);
  }
}

/** Empty condition. Is only possible for toplevel conditions. */
export class ConditionNone extends Condition {}

/** Condition with a single constraint */
export class ConditionSingle extends Condition {
  constructor(public constraint: ConstraintSerialized) {
    super();
  }

  toJSON(): ConditionSerialized {
    return {
      type: ConditionType.Single,
      constraint: this.constraint,
    };
  }
}

/** Condition with multiple joint constraints */
export class ConditionAnd extends Condition {
  constructor(public conditions: Condition[]) {
    super();
  }

  toJSON(): ConditionSerialized {
    return {
      type: ConditionType.And,
      conditions: this.conditions.map((c) => c.toJSON()),
    };
  }
}

/** Condition with multiple alternative constraints */
export class ConditionOr extends Condition {
  constructor(public conditions: Condition[]) {
    super();
  }

  toJSON(): ConditionSerialized {
    return {
      type: ConditionType.Or,
      conditions: this.conditions.map((c) => c.toJSON()),
    };
  }
}
