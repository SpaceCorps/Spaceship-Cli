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
}

[Description("Save DNS records")]
public sealed class SaveCommand : SpaceshipCommand<SaveSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SaveSettings settings)
    {
        string json;
        if (!string.IsNullOrWhiteSpace(settings.File))
            json = await System.IO.File.ReadAllTextAsync(settings.File);
        else if (!System.Console.IsInputRedirected)
            throw new SpaceshipException("Provide records via stdin or --file. Expected JSON: {\"items\": [...]} or [ ... ]");
        else
            json = await System.Console.In.ReadToEndAsync();

        var body = JsonSerializer.Deserialize<JsonElement>(json);
        // API expects {"items": [...]} — wrap bare arrays automatically
        object payload;
        if (body.ValueKind == JsonValueKind.Array)
            payload = new { items = ToObject(body) };
        else
            payload = ToObject(body);
        var result = await client.PutAsync($"/dns/records/{settings.Domain}", payload);
        return ToObject(result);
    }
}
