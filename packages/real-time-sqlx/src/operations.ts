/** Simple SQL operations */

import {
  OperationType,
  type CreateData,
  type GranularOperationCreate,
  type GranularOperationCreateMany,
  type GranularOperationDelete,
  type GranularOperationUpdate,
  type Indexable,
  type UpdateData,
} from "./types";

/** Builder to create an entry in a database */
export const create = <T extends Indexable>(
  table: string,
  data: CreateData<T>,
): GranularOperationCreate<T> => ({
  type: OperationType.Create,
  table,
  data,
});

/** Builder to create many entries in a database */
export const createMany = <T extends Indexable>(
  table: string,
  data: CreateData<T>[],
): GranularOperationCreateMany<T> => ({
  type: OperationType.CreateMany,
  table,
  data,
});

/** Builder to update an entry in a database */
export const update = <T extends Indexable>(
  table: string,
  id: number,
  data: UpdateData<T>,
): GranularOperationUpdate<T> => ({
  type: OperationType.Update,
  id,
  table,
  data,
});

/** Builder to delete an entry in a database */
export const remove = (table: string, id: number): GranularOperationDelete => ({
  type: OperationType.Delete,
  table,
  id,
});
