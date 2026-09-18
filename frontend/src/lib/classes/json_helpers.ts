export function hasRequiredFields(
    json: Record<string, unknown>,
    fields: readonly string[],
): boolean {
    return fields.every(field => Object.hasOwn(json, field));
}
