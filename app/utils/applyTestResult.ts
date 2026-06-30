export function applyTestResult<T extends { id: number }>(
  data: T[],
  updates: Pick<T, 'id'> & Partial<T>
): void {
  const item = data.find(t => t.id === updates.id)
  if (item) Object.assign(item, updates)
}
