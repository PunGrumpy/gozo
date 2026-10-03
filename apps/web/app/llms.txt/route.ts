import { docsLlms } from "@/lib/source";

export const revalidate = false;

// oxlint-disable-next-line sonarjs/function-name -- Next.js route handlers must be named after the HTTP method
export const GET = async () => new Response(await docsLlms.index());
