using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.PersonalNameservers;

public sealed class ListSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name")]
    public required string Domain { get; set; }
}

[Description("List personal nameservers (glue records)")]
public sealed class ListCommand : SpaceshipCommand<ListSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, ListSettings settings)
    {
        var result = await client.GetAsync($"/domains/{settings.Domain}/personal-nameservers");
        return ToObject(result);
    }
}
