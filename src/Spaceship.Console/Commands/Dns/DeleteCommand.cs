using System.ComponentModel;
using System.Text.Json;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Dns;

public sealed class DeleteSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandOption("--file <FILE>")]
    [Description("JSON file with records array to delete (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Delete DNS records")]
public sealed class DeleteCommand : SpaceshipCommand<DeleteSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, DeleteSettings settings)
    {
        string json;
        if (!string.IsNullOrWhiteSpace(settings.File))
            json = await System.IO.File.ReadAllTextAsync(settings.File);
        else if (!System.Console.IsInputRedirected)
            throw new SpaceshipException("Provide records via stdin or --file. Expected JSON: [ ... ] or {\"items\": [...]}");
        else
            json = await System.Console.In.ReadToEndAsync();

        var body = JsonSerializer.Deserialize<JsonElement>(json);
        // Unlike save's PUT, this endpoint expects a bare array — wrapping it in
        // {"items": [...]} is rejected with 422. Accept either shape, always send the array.
        object payload;
        if (body.ValueKind == JsonValueKind.Array)
            payload = ToObject(body);
        else if (body.ValueKind == JsonValueKind.Object
                 && body.TryGetProperty("items", out var items)
                 && items.ValueKind == JsonValueKind.Array)
            payload = ToObject(items);
        else
            throw new SpaceshipException("Expected a JSON array of records, or {\"items\": [...]}.");
        var result = await client.DeleteAsync($"/dns/records/{settings.Domain}", payload);
        return ToObject(result);
    }
}
