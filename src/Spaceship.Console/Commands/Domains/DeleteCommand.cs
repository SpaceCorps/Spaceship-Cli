using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.Domains;

public sealed class DeleteSettings : GlobalSettings
{
    [CommandArgument(0, "<domain>")]
    [Description("Domain name to delete")]
    public required string Domain { get; set; }
}

[Description("Delete a domain (the API currently returns 501 Not Implemented)")]
public sealed class DeleteCommand : SpaceshipCommand<DeleteSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, DeleteSettings settings)
    {
        var result = await client.DeleteAsync($"/domains/{settings.Domain}");
        return ToObject(result);
    }
}
