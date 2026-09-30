import { fingerprint, isWebSignError } from "@websign/sdk";
import { errorText } from "@websign/sdk/messages";
import { setUpFake } from "../shared/fake";
import { samplePdf } from "./pdf";
import { signPdf } from "./sign-pdf";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

await setUpFake();

async function run(input: Uint8Array, name: string) {
  const button = $<HTMLButtonElement>("sample");
  button.disabled = true;
  $("result").hidden = true;
  try {
    const { pdf, certificate, digest } = await signPdf(input);
    const link = document.createElement("a");
    link.href = URL.createObjectURL(new Blob([pdf as BlobPart], { type: "application/pdf" }));
    link.download = `${name.replace(/\.pdf$/i, "")}-signed.pdf`;
    link.textContent = "Download the signed PDF";
    $("download").replaceChildren(link);
    $("summary").textContent =
      `Signed by ${certificate.displayName} (${certificate.issuerName}). Verification code ${fingerprint(digest).text}.`;
    $("result").className = "card";
  } catch (error) {
    if (isWebSignError(error, "UserCancelled")) return;
    const text = isWebSignError(error) ? errorText(error) : undefined;
    $("summary").textContent = text ? `${text.title}. ${text.body}` : String(error);
    $("download").replaceChildren();
    $("result").className = "card error";
  } finally {
    $("result").hidden = false;
    button.disabled = false;
  }
}

$("sample").addEventListener("click", async () => run(await samplePdf(), "sample.pdf"));
$<HTMLInputElement>("file").addEventListener("change", async (event) => {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) await run(new Uint8Array(await file.arrayBuffer()), file.name);
});
