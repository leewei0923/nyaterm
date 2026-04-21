import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { SelectItem } from "@/components/ui/select";
import { useApp } from "@/context/AppContext";
import { invoke } from "@/lib/invoke";
import { NumberInput } from "../ui/number-input";
import {
  SettingInput,
  SettingRow,
  SettingSection,
  SettingSelect,
  SettingSwitch,
} from "./SettingFormItems";

export function SecurityTab() {
  const { t } = useTranslation();
  const { appSettings, updateAppSettings } = useApp();
  const [masterPasswordValue, setMasterPasswordValue] = useState("");

  useEffect(() => {
    if (!appSettings.security.master_password) {
      setMasterPasswordValue("");
      return;
    }

    if (appSettings.security.master_password !== "__SET__") {
      setMasterPasswordValue(appSettings.security.master_password);
      return;
    }

    let cancelled = false;
    invoke<string | null>("get_master_password_value")
      .then((value) => {
        if (!cancelled) {
          setMasterPasswordValue(value ?? "");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMasterPasswordValue("");
        }
      });

    return () => {
      cancelled = true;
    };
  }, [appSettings.security.master_password]);

  return (
    <div className="space-y-5">
      <SettingSection title={t("settings.masterPasswordSection")}>
        <SettingInput
          label={t("settings.masterPassword")}
          desc={t("settings.masterPasswordDesc")}
          type="password"
          controlClassName="max-w-lg"
          placeholder={t("settings.masterPasswordPlaceholder")}
          value={masterPasswordValue}
          onChange={(e) => {
            const val = e.target.value;
            setMasterPasswordValue(val);
            updateAppSettings({
              security: { ...appSettings.security, master_password: val || undefined },
            });
          }}
        />
      </SettingSection>

      <SettingSection title={t("settings.sessionSecurity")} contentClassName="space-y-5">
        <SettingRow
          label={t("settings.enableScreenLock")}
          desc={t("settings.enableScreenLockDesc")}
        >
          <SettingSwitch
            checked={appSettings.security.enable_screen_lock}
            onChange={(v) =>
              updateAppSettings({
                security: { ...appSettings.security, enable_screen_lock: v },
              })
            }
          />
        </SettingRow>

        {appSettings.security.enable_screen_lock && (
          <SettingRow
            label={t("settings.idleLockMinutes")}
            desc={t("settings.idleLockMinutesDesc")}
          >
            <div className="flex w-full max-w-xs items-center gap-3 sm:w-auto">
              <NumberInput
                min={0}
                max={1440}
                className="w-full sm:w-32"
                value={appSettings.security.idle_lock_minutes}
                onChange={(v) =>
                  updateAppSettings({
                    security: { ...appSettings.security, idle_lock_minutes: v || 0 },
                  })
                }
              />
              <span className="shrink-0 text-sm text-muted-foreground">{t("common.minutes")}</span>
            </div>
          </SettingRow>
        )}
      </SettingSection>

      <SettingSection>
        <SettingSelect
          label={t("settings.hostKeyPolicy")}
          desc={t("settings.hostKeyPolicyDesc")}
          value={appSettings.security.host_key_policy}
          controlClassName="max-w-sm"
          onValueChange={(v) =>
            updateAppSettings({ security: { ...appSettings.security, host_key_policy: v } })
          }
        >
          <SelectItem value="strict">{t("settings.hostKeyStrict")}</SelectItem>
          <SelectItem value="prompt">{t("settings.hostKeyPrompt")}</SelectItem>
          <SelectItem value="accept">{t("settings.hostKeyAccept")}</SelectItem>
        </SettingSelect>
      </SettingSection>
    </div>
  );
}
