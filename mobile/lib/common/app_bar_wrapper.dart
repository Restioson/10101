import 'package:flutter/material.dart';
import 'package:get_10101/features/trade/settings_screen.dart';
import 'package:get_10101/features/trade/trade_screen.dart';
import 'package:get_10101/features/wallet/scanner_screen.dart';
import 'package:get_10101/features/wallet/settings_screen.dart';
import 'package:get_10101/features/wallet/wallet_screen.dart';
import 'package:go_router/go_router.dart';

class AppBarWrapper extends StatelessWidget {
  const AppBarWrapper({
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final currentRoute = GoRouter.of(context).location;

    var actionButtons = <Widget>[];
    Widget? leadingButton;

    if (currentRoute == WalletScreen.route) {
      actionButtons = [
        IconButton(
          icon: const Icon(Icons.qr_code_scanner),
          tooltip: 'Scanner',
          onPressed: () {
            context.go(ScannerScreen.route);
          },
        )
      ];

      leadingButton = IconButton(
        icon: const Icon(Icons.settings),
        tooltip: 'Settings',
        onPressed: () {
          context.go(WalletSettingsScreen.route);
        },
      );
    }

    if (currentRoute == TradeScreen.route) {
      leadingButton = IconButton(
        icon: const Icon(Icons.settings),
        tooltip: 'Settings',
        onPressed: () {
          context.go(TradeSettingsScreen.route);
        },
      );
    }

    return AppBar(
      elevation: 0,
      backgroundColor: Colors.transparent,
      iconTheme: const IconThemeData(color: Colors.black),
      leading: leadingButton,
      actions: actionButtons,
    );
  }
}
