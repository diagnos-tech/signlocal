/**
 * The `websign` executable: `WEBSIGN_EXECUTABLE`, else `websign` on PATH,
 * else the install locations of docs/architecture/packaging-and-release.md
 * §Install locations. Undefined when not found.
 */
export function findExecutable(): string | undefined {
  throw new Error("unimplemented: SPEC.md §1");
}
