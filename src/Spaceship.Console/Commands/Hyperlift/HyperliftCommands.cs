using System.ComponentModel;
using System.Text.Json;
using Spaceship.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Hyperlift;

public class AppSettings : GlobalSettings
{
    [CommandArgument(0, "<id>")]
    [Description("Hyperlift application ID")]
    public required string Id { get; set; }
}

public sealed class LogSettings : AppSettings
{
    [CommandOption("--take")]
    [Description("Number of log lines to return (1-100)")]
    [DefaultValue(100)]
    public int Take { get; set; } = 100;

    [CommandOption("--cursor <CURSOR>")]
    [Description("Cursor from the previous page")]
    public string? Cursor { get; set; }

    public override ValidationResult Validate()
    {
        if (Take is < 1 or > 100)
            return ValidationResult.Error("--take must be between 1 and 100.");
        return base.Validate();
    }
}

[Description("List Hyperlift applications")]
public sealed class ListCommand : SpaceshipCommand<PaginatedSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, PaginatedSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build("/hyperlift/applications",
            ("take", settings.Take), ("skip", settings.Skip)));
        return ToObject(result);
    }
}

[Description("Get a Hyperlift application")]
public sealed class GetCommand : SpaceshipCommand<AppSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, AppSettings settings)
    {
        var result = await client.GetAsync($"/hyperlift/applications/{settings.Id}");
        return ToObject(result);
    }
}

[Description("Start a build of a Hyperlift application")]
public sealed class BuildCommand : SpaceshipCommand<AppSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, AppSettings settings)
    {
        var result = await client.PostAsync($"/hyperlift/applications/{settings.Id}/build");
        return ToObject(result);
    }
}

[Description("Get build logs (finished: true once the build is done)")]
public sealed class BuildLogsCommand : SpaceshipCommand<LogSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, LogSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build($"/hyperlift/applications/{settings.Id}/build-logs",
            ("take", settings.Take), ("cursor", settings.Cursor)));
        return ToObject(result);
    }
}

[Description("Get runtime logs")]
public sealed class LogsCommand : SpaceshipCommand<LogSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, LogSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build($"/hyperlift/applications/{settings.Id}/logs",
            ("take", settings.Take), ("cursor", settings.Cursor)));
        return ToObject(result);
    }
}

[Description("Get environment variables")]
public sealed class EnvGetCommand : SpaceshipCommand<AppSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, AppSettings settings)
    {
        var result = await client.GetAsync($"/hyperlift/applications/{settings.Id}/environment");
        return ToObject(result);
    }
}

public sealed class EnvSetSettings : AppSettings
{
    [CommandOption("--file <FILE>")]
    [Description("JSON map of NAME to value, max 20 (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Replace ALL environment variables; any variable left out is deleted")]
public sealed class EnvSetCommand : SpaceshipCommand<EnvSetSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, EnvSetSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File, "{\"NODE_ENV\": \"production\"} or {\"items\": {...}}");
        if (body.ValueKind != JsonValueKind.Object)
            throw new SpaceshipException("Expected a JSON object mapping variable names to values.");

        // The API expects {"items": {NAME: value}}; wrap a bare map automatically
        var payload = body.TryGetProperty("items", out var items) && items.ValueKind == JsonValueKind.Object
            ? ToObject(body)
            : new Dictionary<string, object> { ["items"] = ToObject(body) };
        var result = await client.PutAsync($"/hyperlift/applications/{settings.Id}/environment", payload);
        return ToObject(result);
    }
}

public sealed class MetricsSettings : AppSettings
{
    [CommandOption("--start <DATETIME>")]
    [Description("Start of the range, UTC ISO 8601 (e.g. 2026-01-15T00:00:00Z)")]
    public string? Start { get; set; }

    [CommandOption("--end <DATETIME>")]
    [Description("End of the range, UTC ISO 8601")]
    public string? End { get; set; }

    [CommandOption("--interval <INTERVAL>")]
    [Description("Bucket size: number + s|m|h|d, e.g. 10m (max 1500 buckets per request)")]
    public string? Interval { get; set; }

    [CommandOption("--metrics <METRICS>")]
    [Description("Comma-separated: memoryUsageBytes, cpuUsagePercentage, networkReceiveRateBytes, networkTransmitRateBytes, ephemeralStorageUsedMebibytes, persistentStorageUsedMebibytes")]
    public string? Metrics { get; set; }

    public override ValidationResult Validate()
    {
        if (string.IsNullOrWhiteSpace(Start) || string.IsNullOrWhiteSpace(End)
            || string.IsNullOrWhiteSpace(Interval) || string.IsNullOrWhiteSpace(Metrics))
            return ValidationResult.Error("--start, --end, --interval and --metrics are all required.");
        return base.Validate();
    }
}

[Description("Get application metrics")]
public sealed class MetricsCommand : SpaceshipCommand<MetricsSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, MetricsSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build($"/hyperlift/applications/{settings.Id}/metrics",
            ("startDate", settings.Start), ("endDate", settings.End),
            ("interval", settings.Interval), ("metrics", settings.Metrics)));
        return ToObject(result);
    }
}

[Description("Restart a Hyperlift application")]
public sealed class RestartCommand : SpaceshipCommand<AppSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, AppSettings settings)
    {
        var result = await client.PostAsync($"/hyperlift/applications/{settings.Id}/restart");
        return ToObject(result);
    }
}

public sealed class ScaleSettings : AppSettings
{
    [CommandOption("--scale <SCALE>")]
    [Description("0 stops the application, 1 starts it")]
    public int? Scale { get; set; }

    public override ValidationResult Validate()
    {
        if (Scale is null or < 0 or > 1)
            return ValidationResult.Error("--scale is required: 0 stops the application, 1 starts it.");
        return base.Validate();
    }
}

[Description("Start (1) or stop (0) a Hyperlift application")]
public sealed class ScaleCommand : SpaceshipCommand<ScaleSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, ScaleSettings settings)
    {
        var result = await client.PutAsync($"/hyperlift/applications/{settings.Id}/scale", new { scale = settings.Scale });
        return ToObject(result);
    }
}
