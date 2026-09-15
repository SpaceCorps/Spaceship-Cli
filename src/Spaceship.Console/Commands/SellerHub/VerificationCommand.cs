using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.SellerHub;

[Description("Get SellerHub ownership verification record options")]
public sealed class VerificationCommand : SpaceshipCommand<GlobalSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, GlobalSettings settings)
    {
        var result = await client.GetAsync("/sellerhub/verification-records");
        return ToObject(result);
    }
}
