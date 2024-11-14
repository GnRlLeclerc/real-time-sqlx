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
  Many = "many",
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

export interface ManyQueryData<T> {
  type: QueryReturnType.Many;
  data: T[];
}

/** Generic data return type */
export type QueryData<T> = SingleQueryData<T> | ManyQueryData<T>;

// ************************************************************************* //
//                                  UPDATES                                  //
// ************************************************************************* //

/** Base type for table data which has an index */
export interface Indexable {
  id: FinalConstraintValue;
}

/** Indexable data update type (partial attributes without the id) */
export type UpdateData<T extends Indexable> = Partial<Omit<T, keyof Indexable>>;

type MakeFieldOptional<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

/** Indexable data creation type (all attributes with optional id) */
export type CreateData<T extends Indexable> = MakeFieldOptional<T, "id">;

/** Database operation type */
export enum OperationType {
  Create = "create",
  CreateMany = "create_many",
  Update = "update",
  Delete = "delete",
}

// ************************************************************************* //
//                         GRANULAR UPDATE OPERATIONS                        //
// ************************************************************************* //

// Granular frontend operations
// The frontend sends these operations to be done by the backend

interface GranularOperationBase {
  type: OperationType;
  table: string;
}

/** Create an entry in a database table */
export interface GranularOperationCreate<T extends Indexable>
  extends GranularOperationBase {
  type: OperationType.Create;
  data: CreateData<T>;
}

/** Create many entries in a database table */
export interface GranularOperationCreateMany<T extends Indexable>
  extends GranularOperationBase {
  type: OperationType.CreateMany;
  data: CreateData<T>[];
}

/** Update an entry in a database table */
export interface GranularOperationUpdate<T extends Indexable>
  extends GranularOperationBase {
  type: OperationType.Update;
  id: FinalConstraintValue;
  data: UpdateData<T>;
}

/** Delete an entry in a database table */
export interface GranularOperationDelete extends GranularOperationBase {
  type: OperationType.Delete;
  id: FinalConstraintValue;
}

/** Granular database operation (used in the frontend) */
export type GranularOperation =
  | GranularOperationCreate<any>
  | GranularOperationCreateMany<any>
  | GranularOperationUpdate<any>
  | GranularOperationDelete;

// ************************************************************************* //
//                         OPERATION NOTIFICATIONS                           //
// ************************************************************************* //

// Complete backend operations
// The backend sends back these operation results to be processed by the frontend

interface OperationNotificationBase {
  type: OperationType;
  table: string;
}

/** Notification of entry creation  */
export interface OperationNotificationCreate<T extends Indexable>
  extends OperationNotificationBase {
  type: OperationType.Create;
  data: T; // The full data with ID is sent back
}

/** Notification of multiple entries creation  */
export interface OperationNotificationCreateMany<T extends Indexable>
  extends OperationNotificationBase {
  type: OperationType.CreateMany;
  data: T[]; // The full data with ID is sent back
}

/** Notification of entry update */
export interface OperationNotificationUpdate<T extends Indexable>
  extends OperationNotificationBase {
  type: OperationType.Update;
  id: FinalConstraintValue;
  data: T; // The full data with ID is sent back
}

/** Notification of entry deletion */
export interface OperationNotificationDelete extends OperationNotificationBase {
  type: OperationType.Delete;
  id: FinalConstraintValue;
}

/** Notification of database operation (returned by the backend) */
export type OperationNotification =
  | OperationNotificationCreate<any>
  | OperationNotificationCreateMany<any>
  | OperationNotificationUpdate<any>
  | OperationNotificationDelete;
