/** Query builders for the real-time subscription engine. */

import {
  Condition,
  ConditionAnd,
  ConditionNone,
  ConditionOr,
  ConditionSingle,
} from "./conditions";
import {
  type ConstraintValue,
  type QuerySerialized,
  type QueryOperator,
  QueryReturnType,
  type FinalValue,
} from "./types";

/** Callback to create nested queries */
export type QueryCallback<T> = (
  query: InitialQueryBuilder<T>,
) => BaseQueryBuilder<T>;

// NOTE: we need 2 different final query classes instead of one with generics,
// because typescript is not able to strongly infer the return type with nominal types.

/** Final query class that can be serialized to JSON,
 * for queries that return one value.
 */
export class FinalQuerySingle<T> {
  constructor(
    private _table: string,
    private condition: Condition,
  ) {}

  toJSON(): QuerySerialized {
    return {
      return: QueryReturnType.Single,
      table: this._table,
      condition:
        this.condition instanceof ConditionNone
          ? null
          : this.condition.toJSON(),
    };
  }

  get table(): string {
    return this._table;
  }
}

/** Final query class that can be serialized to JSON,
 * for queries that return multiple values..
 */
export class FinalQueryMany<T> {
  constructor(
    private _table: string,
    private condition: Condition,
  ) {}

  toJSON(): QuerySerialized {
    return {
      return: QueryReturnType.Many,
      table: this._table,
      condition:
        this.condition instanceof ConditionNone
          ? null
          : this.condition.toJSON(),
    };
  }

  get table(): string {
    return this._table;
  }
}

/** Base class for query builders that declares shared data and methods. */
export class BaseQueryBuilder<T> {
  constructor(
    protected table: string,
    protected condition: Condition,
  ) {}

  /** Fetch the first matching row */
  fetchOne(): FinalQuerySingle<T> {
    return new FinalQuerySingle(this.table, this.condition);
  }

  /** Fetch all matching rows */
  fetchMany(): FinalQueryMany<T> {
    return new FinalQueryMany(this.table, this.condition);
  }

  /** Condition accessor for internal use. */
  getCondition(): Condition {
    return this.condition;
  }
}

/** Empty query builder with no conditions */
export class InitialQueryBuilder<T> extends BaseQueryBuilder<T> {
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
    const builder = query(this.table);
    return new QueryBuilderWithCondition(
      this.table,
      callback(builder).getCondition(),
    );
  }
}

/** Query builder with a single condition */
class QueryBuilderWithCondition<T> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: Condition,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  andWhere<C extends keyof T & string, O extends QueryOperator>(
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
  orWhere<C extends keyof T & string, O extends QueryOperator>(
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
  andWhereCallback(
    callback: QueryCallback<T>,
  ): QueryBuilderWithAndCondition<T> {
    const builder = query(this.table);
    const result = callback(builder);

    return QueryBuilderWithAndCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }

  /** Add a new nested alternative condition to the query */
  orWhereCallback(callback: QueryCallback<T>): QueryBuilderWithOrCondition<T> {
    const builder = query(this.table);
    const result = callback(builder);

    return QueryBuilderWithOrCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }
}

/** Query builder with joint conditions */
class QueryBuilderWithAndCondition<T> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: ConditionAnd,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  andWhere<C extends keyof T & string, O extends QueryOperator>(
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
  andWhereCallback(
    callback: QueryCallback<T>,
  ): QueryBuilderWithAndCondition<T> {
    const builder = query(this.table);
    const result = callback(builder);
    this.condition.conditions.push(result.getCondition());

    return this;
  }

  /** Create a new query builder from a list of conditions */
  static fromConditions(table: string, conditions: Condition[]) {
    return new QueryBuilderWithAndCondition(
      table,
      new ConditionAnd(conditions),
    );
  }
}

/** Query builder with alternative conditions */
class QueryBuilderWithOrCondition<T> extends BaseQueryBuilder<T> {
  constructor(
    table: string,
    protected condition: ConditionOr,
  ) {
    super(table, condition);
  }

  /** Add a new alternative condition to the query */
  orWhere<C extends keyof T & string, O extends QueryOperator>(
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
  orWhereCallback(callback: QueryCallback<T>): QueryBuilderWithOrCondition<T> {
    const builder = query(this.table);
    const result = callback(builder);
    this.condition.conditions.push(result.getCondition());

    return this;
  }

  /** Create a new query builder from a list of conditions */
  static fromConditions(table: string, conditions: Condition[]) {
    return new QueryBuilderWithOrCondition(table, new ConditionOr(conditions));
  }
}

/** Create a new query on a table */
export const query = <T>(table: string): InitialQueryBuilder<T> =>
  new InitialQueryBuilder(table);
