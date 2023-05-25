import puppeteer, { type BrowserWorker } from '@cloudflare/puppeteer';

export interface Env {
    IMAGINE: R2Bucket;
    BROWSER: BrowserWorker;
}

function parseQuery(query: any) {
    if (typeof query != 'string' || query?.trim()?.length == 0) {
        return { key: 'imagine.gif', name: 'imagine' };
    }

    const name = query
        .trim()
        .toLowerCase()
        .replace(/[^a-zA-Z0-9!? ]/gm, '');

    if (name == 'imagine') {
        return { key: 'imagine.gif', name: 'imagine' };
    }

    const key = `generated/${name.replace(/ /gm, '-')}.gif`;

    return { key, name };
}

export default {
    async fetch(
        request: Request,
        env: Env,
        ctx: ExecutionContext,
    ): Promise<Response> {
        const url = new URL(request.url);

        if (url.pathname == '/base') {
            const file = await env.IMAGINE.get('base.gif');

            return new Response(file?.body, {
                headers: {
                    'Content-Type': 'image/gif',
                },
            });
        }

        const { key, name } = parseQuery(url.searchParams.get('q'));
        const file = await env.IMAGINE.get(key);

        if (file)
            return new Response(file.body, {
                headers: {
                    'Content-Type': 'image/gif',
                },
            });

        // const browser = await puppeteer.launch(env.BROWSER);

        // const [page] = await browser.pages();
        // await page.goto('https://qifi.org/');
        // await page.type('#ssid', 'homebase');
        // await page.type('#key', 'tubulargagy');
        // await page.click('#generate');

        // const res = await page.$eval('#qrcode canvas', (el) => el.toDataURL());
        // console.log(res);

        return new Response('asd');
    },
};
