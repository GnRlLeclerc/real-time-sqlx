/** Subscription serialized type declarations */

// ************************************************************************* //
//                                CONSTRAINTS                                //
// ************************************************************************* //

type FinalConstraintValue = string | number | boolean | null;

/** Constraint value */
export type ConstraintValue = FinalConstraintValue | FinalConstraintValue[];

/** Constraint data */
export interface ConstraintSerialized {
  column: string;
  operator: QueryOperator;
  value: ConstraintValue;
}

// ************************************************************************* //
//                                CONDITIONS                                 //
// ************************************************************************* //

/** Condition operators */
export type QueryOperator =
  | "="
  | "<"
  | ">"
  | "<="
  | ">="
  | "!="
  | "in"
  | "like"
  | "ilike";

/** Condition type */
export enum ConditionType {
  Single = "single", // One single condition
  And = "and", // A list of conditions
  Or = "or", // A list of conditions
}

/** Condition data */
export type ConditionSerialized =
  | {
      type: ConditionType.Single;
      constraint: ConstraintSerialized;
    }
  | {
      type: ConditionType.And | ConditionType.Or;
      conditions: ConditionSerialized[];
    };

// ************************************************************************* //
//                                 QUERIES                                   //
// ************************************************************************* //

/** How many rows should be returned from the query */
export enum QueryReturnType {
  Single = "single",
  Multiple = "multiple",
}

/** Complete query data */
export interface QuerySerialized {
  return: QueryReturnType;
  table: string;
  condition: ConditionSerialized | null;
}

// ************************************************************************* //
//                                   DATA                                    //
// ************************************************************************* //

export interface SingleQueryData<T> {
  type: QueryReturnType.Single;
  data: T | null;
}

export interface MultipleQueryData<T> {
  type: QueryReturnType.Multiple;
  data: T[];
}

/** Generic data return type */
export type QueryData<T> = SingleQueryData<T> | MultipleQueryData<T>;
