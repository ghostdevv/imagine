export interface Env {
    IMAGINE: R2Bucket;
}

export default {
    async fetch(
        request: Request,
        env: Env,
        ctx: ExecutionContext,
    ): Promise<Response> {
        const file = await env.IMAGINE.get('imagine.gif');

        if (!file)
            return new Response('Not found', {
                status: 404,
            });

        return new Response(await file.blob(), {
            headers: {
                'content-type': file.httpMetadata.contentType!,
            },
        });
    },
};
