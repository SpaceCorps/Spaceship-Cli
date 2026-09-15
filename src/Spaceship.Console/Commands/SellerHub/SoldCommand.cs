using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.SellerHub;

public sealed class SoldSettings : GlobalSettings
{
    [CommandOption("--take")]
    [Description("Number of items to return (1-100)")]
    [DefaultValue(20)]
    public int Take { get; set; } = 20;

    [CommandOption("--cursor <CURSOR>")]
    [Description("Cursor from the previous page")]
    public string? Cursor { get; set; }

    [CommandOption("--from <DATETIME>")]
    [Description("Only sales at or after this time (ISO 8601)")]
    public string? From { get; set; }

    [CommandOption("--to <DATETIME>")]
    [Description("Only sales before this time (ISO 8601)")]
    public string? To { get; set; }

    public override ValidationResult Validate()
    {
        if (Take is < 1 or > 100)
            return ValidationResult.Error("--take must be between 1 and 100.");
        return base.Validate();
    }
}

[Description("List sold SellerHub domains")]
public sealed class SoldCommand : SpaceshipCommand<SoldSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, SoldSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build("/sellerhub/domains/reports/sold",
            ("take", settings.Take), ("cursor", settings.Cursor),
            ("saleDateTimeFrom", settings.From), ("saleDateTimeTo", settings.To)));
        return ToObject(result);
    }
}
