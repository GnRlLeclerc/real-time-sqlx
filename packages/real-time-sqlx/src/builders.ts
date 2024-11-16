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
} from "./types";

/** Callback to create nested queries */
export type QueryCallback = (query: InitialQueryBuilder) => BaseQueryBuilder;

// NOTE: we need 2 different final query classes instead of one with generics,
// because typescript is not able to strongly infer the return type with nominal types.

/** Final query class that can be serialized to JSON,
 * for queries that return one value.
 */
export class FinalQuerySingle {
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
export class FinalQueryMany {
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
export class BaseQueryBuilder {
  constructor(
    protected table: string,
    protected condition: Condition,
  ) {}

  /** Fetch the first matching row */
  fetchOne(): FinalQuerySingle {
    return new FinalQuerySingle(this.table, this.condition);
  }

  /** Fetch all matching rows */
  fetchMany(): FinalQueryMany {
    return new FinalQueryMany(this.table, this.condition);
  }

  /** Condition accessor for internal use. */
  getCondition(): Condition {
    return this.condition;
  }
}

/** Empty query builder with no conditions */
export class InitialQueryBuilder extends BaseQueryBuilder {
  constructor(table: string) {
    super(table, new ConditionNone());
  }

  /** Add a single constraint to the query */
  where(
    column: string,
    operator: QueryOperator,
    value: ConstraintValue,
  ): QueryBuilderWithCondition {
    return new QueryBuilderWithCondition(
      this.table,
      new ConditionSingle({ column, operator, value }),
    );
  }

  /** Add a nested condition to the query */
  whereCallback(callback: QueryCallback): QueryBuilderWithCondition {
    const builder = query(this.table);
    return new QueryBuilderWithCondition(
      this.table,
      callback(builder).getCondition(),
    );
  }
}

/** Query builder with a single condition */
class QueryBuilderWithCondition extends BaseQueryBuilder {
  constructor(
    table: string,
    protected condition: Condition,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  andWhere(
    column: string,
    operator: QueryOperator,
    value: ConstraintValue,
  ): QueryBuilderWithAndCondition {
    return new QueryBuilderWithAndCondition(
      this.table,
      this.condition.and({ column, operator, value }),
    );
  }

  /** Add a new alternative condition to the query */
  orWhere(
    column: string,
    operator: QueryOperator,
    value: ConstraintValue,
  ): QueryBuilderWithOrCondition {
    return new QueryBuilderWithOrCondition(
      this.table,
      this.condition.or({ column, operator, value }),
    );
  }

  /** Add a new nested joint condition to the query */
  andWhereCallback(callback: QueryCallback): QueryBuilderWithAndCondition {
    const builder = query(this.table);
    const result = callback(builder);

    return QueryBuilderWithAndCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }

  /** Add a new nested alternative condition to the query */
  orWhereCallback(callback: QueryCallback): QueryBuilderWithOrCondition {
    const builder = query(this.table);
    const result = callback(builder);

    return QueryBuilderWithOrCondition.fromConditions(this.table, [
      this.condition,
      result.getCondition(),
    ]);
  }
}

/** Query builder with joint conditions */
class QueryBuilderWithAndCondition extends BaseQueryBuilder {
  constructor(
    table: string,
    protected condition: ConditionAnd,
  ) {
    super(table, condition);
  }

  /** Add a new joint condition to the query */
  andWhere(
    column: string,
    operator: QueryOperator,
    value: ConstraintValue,
  ): QueryBuilderWithAndCondition {
    // Push a new ConstraintSingle to the list of conditions
    this.condition.conditions.push(
      Condition.fromConstraint({ column, operator, value }),
    );
    return this;
  }

  /** Add a new nested joint condition to the query */
  andWhereCallback(callback: QueryCallback): QueryBuilderWithAndCondition {
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
class QueryBuilderWithOrCondition extends BaseQueryBuilder {
  constructor(
    table: string,
    protected condition: ConditionOr,
  ) {
    super(table, condition);
  }

  /** Add a new alternative condition to the query */
  orWhere(
    column: string,
    operator: QueryOperator,
    value: ConstraintValue,
  ): QueryBuilderWithOrCondition {
    this.condition.conditions.push(
      Condition.fromConstraint({ column, operator, value }),
    );
    return this;
  }

  /** Add a new nested alternative condition to the query */
  orWhereCallback(callback: QueryCallback): QueryBuilderWithOrCondition {
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
export const query = (table: string): InitialQueryBuilder =>
  new InitialQueryBuilder(table);
