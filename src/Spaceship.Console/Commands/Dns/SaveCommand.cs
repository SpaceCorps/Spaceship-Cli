using System.ComponentModel;
using System.Text.Json;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Dns;

public sealed class SaveSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandOption("--file <FILE>")]
    [Description("JSON file with records array (or pipe via stdin)")]
    public string? File { get; set; }

    [CommandOption("--force")]
    [Description("Turn off the API's conflict checks and force the zone update")]
    public bool Force { get; set; }
}

[Description("Add DNS records, or update the TTL of matching ones")]
public sealed class SaveCommand : SpaceshipCommand<SaveSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SaveSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File, "[ ... ] or {\"items\": [...]}");

        // PUT expects {"items": [...], "force": bool} — wrap bare arrays automatically
        Dictionary<string, object> payload;
        if (body.ValueKind == JsonValueKind.Array)
            payload = new Dictionary<string, object> { ["items"] = ToObject(body) };
        else if (body.ValueKind == JsonValueKind.Object && body.TryGetProperty("items", out _))
            payload = (Dictionary<string, object>)ToObject(body);
        else
            throw new SpaceshipException("Expected a JSON array of records, or {\"items\": [...]}.");

        if (settings.Force)
            payload["force"] = true;

        var result = await client.PutAsync($"/dns/records/{settings.Domain}", payload);
        return ToObject(result);
    }
}
