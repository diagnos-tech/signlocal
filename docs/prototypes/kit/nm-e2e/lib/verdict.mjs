// Judges what the page saw and what the host logged.

/** Text of a host error reply, for messages. */
const describeError = (reply) =>
  reply?.error ? `${reply.error.code}: ${reply.error.message}` : JSON.stringify(reply);

/**
 * @param {object} result   window.__result from page.html
 * @param {string[]} log    host log lines written during this run
 * @param {{ devId: string, expectSign: boolean }} expected
 * @returns {{ ok: boolean, reason?: string, warnings: string[] }}
 */
export function judge(result, log, expected) {
  const warnings = [];
  const fail = (reason) => ({ ok: false, reason, warnings });

  if (!result) return fail("the page produced no result");
  if (result.failure) return fail(result.failure);
  if (!result.announce) return fail("the extension did not announce itself on the page");

  const { status, ping } = result;
  if (status?.state !== "ready") {
    return fail(`extension status is "${status?.state}" (${describeError(status)})`);
  }
  if (!ping?.ok || ping.type !== "pong") return fail(`ping failed: ${describeError(ping)}`);
  if (ping.v !== 1) return fail(`ping reply has protocol version ${ping.v}, expected 1`);
  if (ping.launch?.family !== "chromium" || ping.launch?.extension_id !== expected.devId) {
    return fail(`the browser started the host for ${JSON.stringify(ping.launch)}, expected chromium/${expected.devId}`);
  }

  if (result.wrongDigest?.error?.code !== "digest_length") {
    return fail(`a 31-byte SHA-256 digest was not refused with digest_length: ${describeError(result.wrongDigest)}`);
  }

  // "The app records every extension connection": the ping the extension
  // sends by itself must have reached the host's log.
  const connections = log.filter((line) => line.includes("event=client"));
  if (connections.length === 0) return fail("the host log has no extension connection record");

  const { list, sign } = result;
  if (!list?.ok) {
    if (expected.expectSign) return fail(`list failed: ${describeError(list)}`);
    warnings.push(`list failed (key sources not ready?): ${describeError(list)}`);
  } else {
    for (const warning of list.warnings ?? []) warnings.push(`key source: ${warning}`);
    if (list.certificates.length === 0) {
      if (expected.expectSign) return fail("no certificate was listed");
      warnings.push("no certificates listed");
    }
  }

  if (sign) {
    if (!sign.ok) {
      if (expected.expectSign) return fail(`sign failed: ${describeError(sign)}`);
      warnings.push(`sign failed: ${describeError(sign)}`);
    } else if (sign.verified !== true) {
      return fail("the host returned a signature that does not verify against the certificate");
    }
  } else if (expected.expectSign) {
    return fail("nothing was signed: no certificate can sign");
  }
  return { ok: true, warnings };
}
