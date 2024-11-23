<script lang="ts">
    import { createClient, FetchTransport } from "@rspc/client";
    import type { Procedures } from "../../../back/bind";

    const client = createClient<Procedures>({
        transport: new FetchTransport(
            "http://localhost:3000/rspc",
            (input, init) => {
                const headers = new Headers(init?.headers);
                headers.append(
                    "Authorization",
                    `Bearer ${window.localStorage.getItem("bearer")}`,
                );
                return fetch(input, { ...init, headers });
            },
        ),
    });
    let email = "";
    let password = "";
</script>

<h1>Welcome to SvelteKit</h1>
<p>
    Visit <a href="https://kit.svelte.dev">kit.svelte.dev</a> to read the documentation
</p>
<form
    on:submit|preventDefault={async (e) => {
        const v = await client.mutation(["create_user", { email, password }]);
        console.log(v);
        window.localStorage.setItem(
            "bearer",
            btoa(String.fromCharCode(...v.sessionId)),
        );
    }}
>
    <p>create user</p>
    <input type="text" bind:value={email} />
    <input type="password" bind:value={password} />
    <input type="submit" value="create" />
</form>
{#await client.query(["ping"])}
    <p>loading</p>
{:then res}
    <p>{JSON.stringify(res)}</p>
{:catch error}
    <p>{JSON.stringify(error)}</p>
{/await}
