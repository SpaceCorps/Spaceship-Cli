using System.ComponentModel;
using System.Text.Json;
using Spaceship.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Dns;

public sealed class ListSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }

    [CommandOption("--take")]
    [Description("Records per page (1-500)")]
    [DefaultValue(100)]
    public int Take { get; set; } = 100;

    [CommandOption("--skip")]
    [Description("Number of records to skip")]
    [DefaultValue(0)]
    public int Skip { get; set; }

    [CommandOption("--all")]
    [Description("Fetch every page instead of one")]
    public bool All { get; set; }

    [CommandOption("--order-by <FIELD>")]
    [Description("Sort by: type, -type, name, -name")]
    public string? OrderBy { get; set; }

    [CommandOption("--type <TYPE>")]
    [Description("Only records of this type, e.g. CNAME (filtered locally across all pages)")]
    public string? Type { get; set; }

    [CommandOption("--name <NAME>")]
    [Description("Only records with this name, e.g. www or @ (filtered locally across all pages)")]
    public string? Name { get; set; }

    public override ValidationResult Validate()
    {
        if (Take is < 1 or > 500)
            return ValidationResult.Error("--take must be between 1 and 500.");
        if (Skip < 0)
            return ValidationResult.Error("--skip must be 0 or greater.");
        return base.Validate();
    }
}

[Description("List DNS records")]
public sealed class ListCommand : SpaceshipCommand<ListSettings>
{
    private const int PageSize = 500;

    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, ListSettings settings)
    {
        var path = $"/dns/records/{settings.Domain}";
        var filtering = !string.IsNullOrWhiteSpace(settings.Type) || !string.IsNullOrWhiteSpace(settings.Name);

        if (!settings.All && !filtering)
        {
            var single = await client.GetAsync(QueryString.Build(path,
                ("take", settings.Take), ("skip", settings.Skip), ("orderBy", settings.OrderBy)));
            return ToObject(single);
        }

        // The API has no type/name filters, so page through the whole zone and filter here.
        var records = new List<JsonElement>();
        var skip = settings.Skip;
        while (true)
        {
            var page = await client.GetAsync(QueryString.Build(path,
                ("take", PageSize), ("skip", skip), ("orderBy", settings.OrderBy)));
            var batch = page.TryGetProperty("items", out var items) && items.ValueKind == JsonValueKind.Array
                ? items.EnumerateArray().ToList()
                : new List<JsonElement>();
            records.AddRange(batch);
            skip += batch.Count;
            var total = page.TryGetProperty("total", out var t) && t.TryGetInt32(out var n) ? n : skip;
            if (batch.Count == 0 || skip >= total)
                break;
        }

        var matches = records
            .Where(r => Matches(r, "type", settings.Type) && Matches(r, "name", settings.Name))
            .ToList();

        return new Dictionary<string, object>
        {
            ["items"] = matches.Select(ToObject).ToList(),
            ["total"] = matches.Count
        };
    }

    private static bool Matches(JsonElement record, string property, string? expected) =>
        string.IsNullOrWhiteSpace(expected)
        || (record.TryGetProperty(property, out var value)
            && string.Equals(value.GetString(), expected, StringComparison.OrdinalIgnoreCase));
}
