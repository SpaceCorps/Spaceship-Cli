using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.SellerHub;

public sealed class UpdateSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Listed domain name")]
    public required string Domain { get; set; }

    [CommandOption("--file <FILE>")]
    [Description("JSON file with fields to update (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Update SellerHub domain")]
public sealed class UpdateCommand : SpaceshipCommand<UpdateSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, UpdateSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File, "{\"displayName\": \"...\", ...}");
        var result = await client.PatchAsync($"/sellerhub/domains/{settings.Domain}", ToObject(body));
        return ToObject(result);
    }
}
