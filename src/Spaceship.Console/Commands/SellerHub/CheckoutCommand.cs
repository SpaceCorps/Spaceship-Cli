using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.SellerHub;

public sealed class CheckoutSettings : GlobalSettings
{
    [CommandOption("--file <FILE>")]
    [Description("JSON file with checkout details (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Create SellerHub checkout link")]
public sealed class CheckoutCommand : SpaceshipCommand<CheckoutSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, CheckoutSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File, "{\"type\": \"buyNow\", \"domainName\": \"...\", ...}");
        var result = await client.PostAsync("/sellerhub/checkout-links", ToObject(body));
        return ToObject(result);
    }
}
