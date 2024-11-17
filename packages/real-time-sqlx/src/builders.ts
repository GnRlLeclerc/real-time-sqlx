/** Query builders for the real-time subscription engine. */

import { invoke } from "@tauri-apps/api/core";
import {
  Condition,
  ConditionAnd,
  ConditionNone,
  ConditionOr,
  ConditionSingle,
} from "./conditions";
import type { UnsubscribeFn, UpdateManyFn, UpdateSingleFn } from "./subscribe";
import { subscribeMany, subscribeOne } from "./subscribe";
import {
  QueryReturnType,
  type FinalValue,
  type Indexable,
  type ManyQueryData,
  type QueryOperator,
  type SerializedQuery,
  type SingleQueryData,
} from "./types";

/** Callback to create nested queries */
export type QueryCallback<T extends Indexable> = (
  query: InitialQueryBuilder<T>,
) => BaseQueryBuilder<T>;

/** Base class for query builders that declares shared data and methods. */
export class BaseQueryBuilder<T extends Indexable> {
  constructor(
    protected table: string,
    protected condition: Condition,
  ) {}

  /** Fetch the first matching row */
  async fetchOne(): Promise<SingleQueryData<T>> {
    const query: SerializedQuery = {
      return: QueryReturnType.Single,
      table: this.table,
      condition:
        this.condition instanceof ConditionNone
          ? null
          : this.condition.toJSON(),
    };
    return await invoke("fetch", { query });
  }

  /** Fetch all matching rows */
  async fetchMany(): Promise<ManyQueryData<T>> {
    const query: SerializedQuery = {
      return: QueryReturnType.Many,
      table: this.table,
      condition:
        this.condition instanceof ConditionNone
          ? null
          : this.condition.toJSON(),
    };
    return await invoke("fetch", { query });
  }

  /** Subscribe to the first matching row */
  subscribeOne(callback: UpdateSingleFn<T>): UnsubscribeFn {
    return subscribeOne(this.table, this.condition, callback);
  }

  /** Subscribe to all matching rows */
  subscribeMany(callback: UpdateManyFn<T>): UnsubscribeFn {
    return subscribeMany(this.table, this.condition, callback);
  }

  /** Condition accessor for internal use. */
  getCondition(): Condition {
    return this.condition;
  }
}

/** Empty query builder with no conditions */
export class InitialQueryBuilder<
  T extends Indexable,
> extends BaseQueryBuilder<T> {
  constructor(table: string) {
    super(table, new ConditionNone());
  }

  /** Add a single constraint to the query */
  where<C extends keyof T & string, O extends QueryOperator>(
    column: C,
    operator: O,
    value: O extends "in" ? (T[C] & FinalValue)[] : T[C] & FinalValue,
  ): QueryBuilderWithCondition<T> {
    return new QueryBuilderWithCondition(
      this.table,
      new ConditionSingle({ column, operator, value }),
    );
  }

  /** Add a nested condition to the query */
  whereCallback(callback: QueryCallback<T>): QueryBuilderWithCondition<T> {
    const builder = query<T>(this.table);
    return new QueryBuilderWithCondition<T>(
      this.table,
      callback(builder).getCondition(),
    );
  }
}

/** Query builder with a single condition */
class QueryBuilderWithCondition<
  T extends Indexable,
> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: Condition,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  and<C extends keyof T & string, O extends QueryOperator>(
    column: C,
    operator: O,
    value: O extends "in" ? (T[C] & FinalValue)[] : T[C] & FinalValue,
  ): QueryBuilderWithAndCondition<T> {
    return new QueryBuilderWithAndCondition<T>(
      this.table,
      this.condition.and({ column, operator, value }),
    );
  }

  /** Add a new alternative condition to the query */
  or<C extends keyof T & string, O extends QueryOperator>(
    column: C,
    operator: O,
    value: O extends "in" ? (T[C] & FinalValue)[] : T[C] & FinalValue,
  ): QueryBuilderWithOrCondition<T> {
    return new QueryBuilderWithOrCondition<T>(
      this.table,
      this.condition.or({ column, operator, value }),
    );
  }

  /** Add a new nested joint condition to the query */
  andCallback(callback: QueryCallback<T>): QueryBuilderWithAndCondition<T> {
    const builder = query<T>(this.table);
    const result = callback(builder);

    return QueryBuilderWithAndCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }

  /** Add a new nested alternative condition to the query */
  orCallback(callback: QueryCallback<T>): QueryBuilderWithOrCondition<T> {
    const builder = query<T>(this.table);
    const result = callback(builder);

    return QueryBuilderWithOrCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }
}

/** Query builder with joint conditions */
class QueryBuilderWithAndCondition<
  T extends Indexable,
> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: ConditionAnd,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  and<C extends keyof T & string, O extends QueryOperator>(
    column: C,
    operator: O,
    value: O extends "in" ? (T[C] & FinalValue)[] : T[C] & FinalValue,
  ): QueryBuilderWithAndCondition<T> {
    // Push a new ConstraintSingle to the list of conditions
    this.condition.conditions.push(
      Condition.fromConstraint({ column, operator, value }),
    );
    return this;
  }

  /** Add a new nested joint condition to the query */
  andCallback(callback: QueryCallback<T>): QueryBuilderWithAndCondition<T> {
    const builder = query<T>(this.table);
    const result = callback(builder);
    this.condition.conditions.push(result.getCondition());

    return this;
  }

  /** Create a new query builder from a list of conditions */
  static fromConditions<T extends Indexable>(
    table: string,
    conditions: Condition[],
  ) {
    return new QueryBuilderWithAndCondition<T>(
      table,
      new ConditionAnd(conditions),
    );
  }
}

/** Query builder with alternative conditions */
class QueryBuilderWithOrCondition<
  T extends Indexable,
> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: ConditionOr,
  ) {
    super(table, condition);
  }

  /** Add a new alternative condition to the query */
  or<C extends keyof T & string, O extends QueryOperator>(
    column: C,
    operator: O,
    value: O extends "in" ? (T[C] & FinalValue)[] : T[C] & FinalValue,
  ): QueryBuilderWithOrCondition<T> {
    this.condition.conditions.push(
      Condition.fromConstraint({ column, operator, value }),
    );
    return this;
  }

  /** Add a new nested alternative condition to the query */
  orCallback(callback: QueryCallback<T>): QueryBuilderWithOrCondition<T> {
    const builder = query<T>(this.table);
    const result = callback(builder);
    this.condition.conditions.push(result.getCondition());

    return this;
  }

  /** Create a new query builder from a list of conditions */
  static fromConditions<T extends Indexable>(
    table: string,
    conditions: Condition[],
  ) {
    return new QueryBuilderWithOrCondition<T>(
      table,
      new ConditionOr(conditions),
    );
  }
}

/** Create a new query on a table.
 * Duplicated here but not exported,
 * without type checking for internal use.
 */
const query = <T extends Indexable>(table: string): InitialQueryBuilder<T> =>
  new InitialQueryBuilder(table);
